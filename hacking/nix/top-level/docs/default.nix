{ lib, pkgs }:

let
  metaCrateName = "meta";

  views =
    let
      f = { world, runtime, minimal }:
        let
          view = world.docs.mkView { inherit runtime minimal; };
        in view // {
          id = lib.substring 0 10 (builtins.hashString "sha256" (view.rustdoc.outPath));
        };
    in map f [
      { world = pkgs.host.aarch64.none.this.worlds.default;
        runtime = null;
        minimal = true;
      }
      { world = pkgs.host.aarch64.none.this.worlds.default;
        runtime = "sel4-root-task-runtime";
        minimal = true;
      }
      { world = pkgs.host.aarch64.none.this.worlds.default;
        runtime = "sel4-root-task-runtime";
        minimal = false;
      }
      { world = pkgs.host.aarch64.none.this.worlds.qemu-arm-virt.sel4cp;
        runtime = "sel4cp";
        minimal = true;
      }
      { world = pkgs.host.aarch64.none.this.worlds.qemu-arm-virt.sel4cp;
        runtime = "sel4cp";
        minimal = false;
      }
      { world = pkgs.host.x86_64.none.this.worlds.default;
        runtime = "sel4-root-task-runtime";
        minimal = false;
      }
      { world = pkgs.host.riscv64.none.this.worlds.default;
        runtime = null;
        minimal = true;
      }
    ];

  mk = { views }: rec {

    html = rustdocHtml;

    # html = pkgs.build.linkFarm "top-level-html" [
    #   { name = "rustdoc"; path = rustdocHtml; }
    # ];

    rustdocHtml = pkgs.build.runCommand "rustdoc-html" {} ''
      mkdir $out
      cp -L ${rustdocHtml'}/index.html $out
      ln -s ${rustdocHtml'}/views $out
    '';

    rustdocHtml' = pkgs.build.linkFarm "rustdoc-html" ([
      { name = "index.html";
        path = rustdocIndex;
      }
    ] ++ lib.forEach views (view: {
      name = "views/${view.id}";
      path = view.rustdoc;
    }));

    rustdocIndex = pkgs.build.runCommand "index.html" {} ''
      substitute ${rustdocIndexIn} $out \
        --replace @content@ "$(cat ${rustdocIndexContent})"
    '';

    rustdocIndexContent = pkgs.build.runCommand "index.content.html" {
      nativeBuildInputs = with pkgs.build; [
        pandoc
      ];
    } ''
      pandoc ${./index.md} -o $out
    '';

    rustdocIndexIn = pkgs.build.writeText "index.html.in" ''
      <!DOCTYPE html>
      <html>
        <head>
          <meta charset="utf-8">
          <meta name="viewport" content="width=device-width, initial-scale=1">
          <title>Rustdoc for rust-seL4</title>
          <link
            rel="stylesheet"
            href="https://cdnjs.cloudflare.com/ajax/libs/github-markdown-css/5.0.0/github-markdown-light.min.css"
            integrity="sha512-2ZxkJRe/dlKUknBZJNP93bh08JvvuvL+fR6I3IqZ4tnAvNQ0D56+LVg+DvE/S/Ir4J/6lxBu/Xye1z243BEa1Q=="
            crossorigin="anonymous"
            referrerpolicy="no-referrer"
          />
          <style>
            .markdown-body {
              box-sizing: border-box;
              min-width: 200px;
              max-width: 980px;
              margin: 0 auto;
              padding: 45px;
            }
            @media (max-width: 767px) {
              .markdown-body {
                padding: 15px;
              }
            }
            ul.xxx {
              margin-bottom: 0;
            }
          </style>
        </head>
        <body>
          <div class="markdown-body">
            <h1>Rustdoc for rust-seL4</h1>
            @content@
            <h3>Views</h3>
            <p>
              <table>
                ${lib.concatStrings
                  (lib.forEach views (view: ''
                    <tr>
                      <td>
                        <ul class=xxx>
                          ${lib.concatStrings [
                            (mkEntry ''
                              <code>PLAT</code>: <code>${view.PLAT}</code>
                            '')
                            (mkEntry ''
                              <code>SEL4_ARCH</code>: <code>${view.SEL4_ARCH}</code>
                            '')
                            (mkEntry ''
                              <code>KERNEL_MCS</code>: <code>${showBool view.KERNEL_MCS}</code>
                            '')
                            (mkEntry ''
                              runtime: ${if view.runtime == null then "(none)" else "<code>${view.runtime}</code>"}
                            '')
                            (mkEntry ''
                              rustc target spec:
                              <a href="${mkJSONDataURI view.targetJSON}">${view.targetName}.json</a>
                            '')
                            (mkEntry ''
                              <a href="./views/${view.id}/${view.targetName}/doc/${metaCrateName}/index.html">rustdoc</a>
                            '')
                            (mkEntry ''
                              <a href="${mkJSONDataURI view.seL4ConfigJSON}">(full seL4 config)</a>
                            '')
                          ]}
                        </ul>
                      </td>
                    </tr>
                  ''))
                }
              </table>
            </p>
          </div>
        </body>
      </html>
    '';
  };

  mkEntry = x: ''
    <li>
      ${x}
    </li>
  '';

  mkJSONDataURI = file:
    let
      b64Drv = pkgs.build.runCommand "x" {} ''
        base64 < ${file} > $out
      '';
      b64 = builtins.readFile b64Drv;
    in
      "data:application/json;base64,${b64}";

  showBool = x: if x then "true" else "false";

in rec {

  realized = mk { inherit views; };

  inherit (realized) html;

  htmlCopied = pkgs.build.runCommand "html" {} ''
    cp -rL ${realized.html} $out
  '';

}
