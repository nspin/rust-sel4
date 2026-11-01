//
// Copyright 2023, Colias Group, LLC
//
// SPDX-License-Identifier: BSD-2-Clause
//

use alloc::borrow::ToOwned;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;

use embedded_io_async::{Read, ReadExactError, Write};

use embedded_fatfs as fat;

use crate::mime::content_type_from_name;

pub(crate) type FileSystem<IO> =
    fat::FileSystem<IO, fat::DefaultTimeProvider, fat::LossyOemCpConverter>;

type File<'a, IO> = fat::File<'a, IO, fat::DefaultTimeProvider, fat::LossyOemCpConverter>;

pub(crate) struct Server<IO: fat::ReadWriteSeek> {
    fs: FileSystem<IO>,
}

impl<IO: fat::ReadWriteSeek> Server<IO> {
    pub(crate) fn new(fs: FileSystem<IO>) -> Self {
        Self { fs }
    }

    pub(crate) async fn handle_connection<U: Read + Write + Unpin>(
        &self,
        conn: &mut U,
    ) -> Result<(), ReadExactError<U::Error>> {
        loop {
            let mut buf = vec![0; 1024 * 16];
            let mut i = 0;
            loop {
                let n = conn.read(&mut buf[i..]).await?;
                if n == 0 {
                    return Err(ReadExactError::UnexpectedEof);
                }
                i += n;
                if is_request_complete(&buf[..i]).unwrap_or(false) {
                    break;
                }
            }
            let mut headers = [httparse::EMPTY_HEADER; 32];
            let mut req = httparse::Request::new(&mut headers);
            let mut keep_alive = false;
            match req.parse(&buf) {
                Ok(status) => {
                    assert!(status.is_complete());
                    self.handle_request(conn, req.path.unwrap()).await?;
                    if should_keep_alive(&req) {
                        keep_alive = true;
                    }
                }
                Err(err) => {
                    log::warn!("error parsing request: {err:?}");
                }
            }
            if !keep_alive {
                break;
            }
        }
        Ok(())
    }

    async fn handle_request<U: Read + Write + Unpin>(
        &self,
        conn: &mut U,
        request_path: &str,
    ) -> Result<(), ReadExactError<U::Error>> {
        match self.lookup_request_path(request_path).await {
            RequestPathStatus::Ok {
                file_name,
                file,
                file_len,
            } => {
                let content_type = content_type_from_name(&file_name);
                self.serve_file(conn, content_type, file, file_len).await?;
            }
            RequestPathStatus::MovedPermanently { location } => {
                self.serve_moved_permanently(conn, &location).await?;
            }
            RequestPathStatus::NotFound => {
                self.serve_not_found(conn).await?;
            }
        }
        Ok(())
    }

    async fn serve_file<U: Write + Unpin>(
        &self,
        conn: &mut U,
        content_type: &str,
        mut file: File<'_, IO>,
        file_len: u64,
    ) -> Result<(), ReadExactError<U::Error>> {
        let file_len: usize = file_len.try_into().unwrap();
        self.start_response_headers(conn, 200, "OK").await?;
        self.send_response_header(conn, "Content-Type", content_type.as_bytes())
            .await?;
        self.send_response_header(conn, "Content-Length", file_len.to_string().as_bytes())
            .await?;
        self.finish_response_headers(conn).await?;
        {
            let mut buf = vec![0; 2048];
            let mut pos = 0;
            while pos < file_len {
                let n = buf.len().min(file_len - pos);
                file.read_exact(&mut buf[..n]).await.unwrap();
                conn.write_all(&buf[..n]).await?;
                pos += n;
            }
        }
        assert_eq!(file.read(&mut [0]).await.unwrap(), 0);
        file.close().await.unwrap();
        Ok(())
    }

    async fn serve_moved_permanently<U: Write + Unpin>(
        &self,
        conn: &mut U,
        location: &str,
    ) -> Result<(), ReadExactError<U::Error>> {
        let phrase = "Moved Permanently";
        self.start_response_headers(conn, 301, phrase).await?;
        self.send_response_header(conn, "Content-Type", b"text/plain")
            .await?;
        self.send_response_header(conn, "Content-Length", phrase.len().to_string().as_bytes())
            .await?;
        self.send_response_header(conn, "Location", location.as_bytes())
            .await?;
        self.finish_response_headers(conn).await?;
        conn.write_all(phrase.as_bytes()).await?;
        Ok(())
    }

    async fn serve_not_found<U: Write + Unpin>(
        &self,
        conn: &mut U,
    ) -> Result<(), ReadExactError<U::Error>> {
        let phrase = "Not Found";
        self.start_response_headers(conn, 404, phrase).await?;
        self.send_response_header(conn, "Content-Type", b"text/plain")
            .await?;
        self.send_response_header(conn, "Content-Length", phrase.len().to_string().as_bytes())
            .await?;
        self.finish_response_headers(conn).await?;
        conn.write_all(phrase.as_bytes()).await?;
        Ok(())
    }

    async fn start_response_headers<U: Write + Unpin>(
        &self,
        conn: &mut U,
        status_code: usize,
        reason_phrase: &str,
    ) -> Result<(), ReadExactError<U::Error>> {
        conn.write_all(b"HTTP/1.1 ").await?;
        conn.write_all(status_code.to_string().as_bytes()).await?;
        conn.write_all(b" ").await?;
        conn.write_all(reason_phrase.as_bytes()).await?;
        conn.write_all(b"\r\n").await?;
        Ok(())
    }

    async fn send_response_header<U: Write + Unpin>(
        &self,
        conn: &mut U,
        name: &str,
        value: &[u8],
    ) -> Result<(), ReadExactError<U::Error>> {
        conn.write_all(name.as_bytes()).await?;
        conn.write_all(b": ").await?;
        conn.write_all(value).await?;
        conn.write_all(b"\r\n").await?;
        Ok(())
    }

    async fn finish_response_headers<U: Write + Unpin>(
        &self,
        conn: &mut U,
    ) -> Result<(), ReadExactError<U::Error>> {
        conn.write_all(b"\r\n").await?;
        Ok(())
    }

    async fn lookup_request_path(&self, request_path: &str) -> RequestPathStatus<File<'_, IO>> {
        if !request_path.starts_with('/') {
            return RequestPathStatus::NotFound;
        }
        let has_trailing_slash = request_path.ends_with('/');
        let mut cur = self.fs.root_dir();
        for seg in request_path.split('/') {
            if seg.is_empty() {
                continue;
            }
            match cur.open_meta(seg).await {
                Ok(entry) => {
                    if entry.is_dir() {
                        cur = entry.to_dir();
                    } else {
                        return RequestPathStatus::Ok {
                            file_name: seg.to_owned(),
                            file: entry.to_file(),
                            file_len: entry.len(),
                        };
                    }
                }
                Err(err) => {
                    log::warn!("{err:?}: {request_path:?}");
                    return RequestPathStatus::NotFound; // TODO
                }
            }
        }
        if !has_trailing_slash {
            RequestPathStatus::MovedPermanently {
                location: format!("{request_path}/"),
            }
        } else {
            let file_name = "index.html";
            let entry = cur.open_meta(file_name).await.unwrap();
            RequestPathStatus::Ok {
                file_name: file_name.to_owned(),
                file: entry.to_file(),
                file_len: entry.len(),
            }
        }
    }
}

enum RequestPathStatus<F> {
    Ok {
        file_name: String,
        file: F,
        file_len: u64,
    },
    MovedPermanently {
        location: String,
    },
    NotFound,
}

fn is_request_complete(buf: &[u8]) -> Result<bool, httparse::Error> {
    let mut headers = [httparse::EMPTY_HEADER; 32];
    let mut req = httparse::Request::new(&mut headers);
    req.parse(buf).map(|status| status.is_complete())
}

fn should_keep_alive(req: &httparse::Request) -> bool {
    let version = req.version.unwrap();
    let default = match version {
        0 => false,
        1 => true,
        _ => panic!(),
    };
    for header in req.headers.iter() {
        if header.name.to_lowercase() == "Connection" {
            if header.value == b"close" {
                return false;
            }
            if header.value == b"keep-alive" {
                return true;
            }
            panic!();
        }
    }
    default
}
