// https://git.musl-libc.org/cgit/musl/tree/arch/x86_64/bits/syscall.h.in

pub mod syscall_number {
    use crate::SyscallNumber;

    pub const READ: SyscallNumber = 0;
    pub const WRITE: SyscallNumber = 1;
    pub const OPEN: SyscallNumber = 2;
    pub const CLOSE: SyscallNumber = 3;
    pub const STAT: SyscallNumber = 4;
    pub const FSTAT: SyscallNumber = 5;
    pub const LSTAT: SyscallNumber = 6;
    pub const POLL: SyscallNumber = 7;
    pub const LSEEK: SyscallNumber = 8;
    pub const MMAP: SyscallNumber = 9;
    pub const MPROTECT: SyscallNumber = 10;
    pub const MUNMAP: SyscallNumber = 11;
    pub const BRK: SyscallNumber = 12;
    pub const RT_SIGACTION: SyscallNumber = 13;
    pub const RT_SIGPROCMASK: SyscallNumber = 14;
    pub const RT_SIGRETURN: SyscallNumber = 15;
    pub const IOCTL: SyscallNumber = 16;
    pub const PREAD64: SyscallNumber = 17;
    pub const PWRITE64: SyscallNumber = 18;
    pub const READV: SyscallNumber = 19;
    pub const WRITEV: SyscallNumber = 20;
    pub const ACCESS: SyscallNumber = 21;
    pub const PIPE: SyscallNumber = 22;
    pub const SELECT: SyscallNumber = 23;
    pub const SCHED_YIELD: SyscallNumber = 24;
    pub const MREMAP: SyscallNumber = 25;
    pub const MSYNC: SyscallNumber = 26;
    pub const MINCORE: SyscallNumber = 27;
    pub const MADVISE: SyscallNumber = 28;
    pub const SHMGET: SyscallNumber = 29;
    pub const SHMAT: SyscallNumber = 30;
    pub const SHMCTL: SyscallNumber = 31;
    pub const DUP: SyscallNumber = 32;
    pub const DUP2: SyscallNumber = 33;
    pub const PAUSE: SyscallNumber = 34;
    pub const NANOSLEEP: SyscallNumber = 35;
    pub const GETITIMER: SyscallNumber = 36;
    pub const ALARM: SyscallNumber = 37;
    pub const SETITIMER: SyscallNumber = 38;
    pub const GETPID: SyscallNumber = 39;
    pub const SENDFILE: SyscallNumber = 40;
    pub const SOCKET: SyscallNumber = 41;
    pub const CONNECT: SyscallNumber = 42;
    pub const ACCEPT: SyscallNumber = 43;
    pub const SENDTO: SyscallNumber = 44;
    pub const RECVFROM: SyscallNumber = 45;
    pub const SENDMSG: SyscallNumber = 46;
    pub const RECVMSG: SyscallNumber = 47;
    pub const SHUTDOWN: SyscallNumber = 48;
    pub const BIND: SyscallNumber = 49;
    pub const LISTEN: SyscallNumber = 50;
    pub const GETSOCKNAME: SyscallNumber = 51;
    pub const GETPEERNAME: SyscallNumber = 52;
    pub const SOCKETPAIR: SyscallNumber = 53;
    pub const SETSOCKOPT: SyscallNumber = 54;
    pub const GETSOCKOPT: SyscallNumber = 55;
    pub const CLONE: SyscallNumber = 56;
    pub const FORK: SyscallNumber = 57;
    pub const VFORK: SyscallNumber = 58;
    pub const EXECVE: SyscallNumber = 59;
    pub const EXIT: SyscallNumber = 60;
    pub const WAIT4: SyscallNumber = 61;
    pub const KILL: SyscallNumber = 62;
    pub const UNAME: SyscallNumber = 63;
    pub const SEMGET: SyscallNumber = 64;
    pub const SEMOP: SyscallNumber = 65;
    pub const SEMCTL: SyscallNumber = 66;
    pub const SHMDT: SyscallNumber = 67;
    pub const MSGGET: SyscallNumber = 68;
    pub const MSGSND: SyscallNumber = 69;
    pub const MSGRCV: SyscallNumber = 70;
    pub const MSGCTL: SyscallNumber = 71;
    pub const FCNTL: SyscallNumber = 72;
    pub const FLOCK: SyscallNumber = 73;
    pub const FSYNC: SyscallNumber = 74;
    pub const FDATASYNC: SyscallNumber = 75;
    pub const TRUNCATE: SyscallNumber = 76;
    pub const FTRUNCATE: SyscallNumber = 77;
    pub const GETDENTS: SyscallNumber = 78;
    pub const GETCWD: SyscallNumber = 79;
    pub const CHDIR: SyscallNumber = 80;
    pub const FCHDIR: SyscallNumber = 81;
    pub const RENAME: SyscallNumber = 82;
    pub const MKDIR: SyscallNumber = 83;
    pub const RMDIR: SyscallNumber = 84;
    pub const CREAT: SyscallNumber = 85;
    pub const LINK: SyscallNumber = 86;
    pub const UNLINK: SyscallNumber = 87;
    pub const SYMLINK: SyscallNumber = 88;
    pub const READLINK: SyscallNumber = 89;
    pub const CHMOD: SyscallNumber = 90;
    pub const FCHMOD: SyscallNumber = 91;
    pub const CHOWN: SyscallNumber = 92;
    pub const FCHOWN: SyscallNumber = 93;
    pub const LCHOWN: SyscallNumber = 94;
    pub const UMASK: SyscallNumber = 95;
    pub const GETTIMEOFDAY: SyscallNumber = 96;
    pub const GETRLIMIT: SyscallNumber = 97;
    pub const GETRUSAGE: SyscallNumber = 98;
    pub const SYSINFO: SyscallNumber = 99;
    pub const TIMES: SyscallNumber = 100;
    pub const PTRACE: SyscallNumber = 101;
    pub const GETUID: SyscallNumber = 102;
    pub const SYSLOG: SyscallNumber = 103;
    pub const GETGID: SyscallNumber = 104;
    pub const SETUID: SyscallNumber = 105;
    pub const SETGID: SyscallNumber = 106;
    pub const GETEUID: SyscallNumber = 107;
    pub const GETEGID: SyscallNumber = 108;
    pub const SETPGID: SyscallNumber = 109;
    pub const GETPPID: SyscallNumber = 110;
    pub const GETPGRP: SyscallNumber = 111;
    pub const SETSID: SyscallNumber = 112;
    pub const SETREUID: SyscallNumber = 113;
    pub const SETREGID: SyscallNumber = 114;
    pub const GETGROUPS: SyscallNumber = 115;
    pub const SETGROUPS: SyscallNumber = 116;
    pub const SETRESUID: SyscallNumber = 117;
    pub const GETRESUID: SyscallNumber = 118;
    pub const SETRESGID: SyscallNumber = 119;
    pub const GETRESGID: SyscallNumber = 120;
    pub const GETPGID: SyscallNumber = 121;
    pub const SETFSUID: SyscallNumber = 122;
    pub const SETFSGID: SyscallNumber = 123;
    pub const GETSID: SyscallNumber = 124;
    pub const CAPGET: SyscallNumber = 125;
    pub const CAPSET: SyscallNumber = 126;
    pub const RT_SIGPENDING: SyscallNumber = 127;
    pub const RT_SIGTIMEDWAIT: SyscallNumber = 128;
    pub const RT_SIGQUEUEINFO: SyscallNumber = 129;
    pub const RT_SIGSUSPEND: SyscallNumber = 130;
    pub const SIGALTSTACK: SyscallNumber = 131;
    pub const UTIME: SyscallNumber = 132;
    pub const MKNOD: SyscallNumber = 133;
    pub const USELIB: SyscallNumber = 134;
    pub const PERSONALITY: SyscallNumber = 135;
    pub const USTAT: SyscallNumber = 136;
    pub const STATFS: SyscallNumber = 137;
    pub const FSTATFS: SyscallNumber = 138;
    pub const SYSFS: SyscallNumber = 139;
    pub const GETPRIORITY: SyscallNumber = 140;
    pub const SETPRIORITY: SyscallNumber = 141;
    pub const SCHED_SETPARAM: SyscallNumber = 142;
    pub const SCHED_GETPARAM: SyscallNumber = 143;
    pub const SCHED_SETSCHEDULER: SyscallNumber = 144;
    pub const SCHED_GETSCHEDULER: SyscallNumber = 145;
    pub const SCHED_GET_PRIORITY_MAX: SyscallNumber = 146;
    pub const SCHED_GET_PRIORITY_MIN: SyscallNumber = 147;
    pub const SCHED_RR_GET_INTERVAL: SyscallNumber = 148;
    pub const MLOCK: SyscallNumber = 149;
    pub const MUNLOCK: SyscallNumber = 150;
    pub const MLOCKALL: SyscallNumber = 151;
    pub const MUNLOCKALL: SyscallNumber = 152;
    pub const VHANGUP: SyscallNumber = 153;
    pub const MODIFY_LDT: SyscallNumber = 154;
    pub const PIVOT_ROOT: SyscallNumber = 155;
    pub const _SYSCTL: SyscallNumber = 156;
    pub const PRCTL: SyscallNumber = 157;
    pub const ARCH_PRCTL: SyscallNumber = 158;
    pub const ADJTIMEX: SyscallNumber = 159;
    pub const SETRLIMIT: SyscallNumber = 160;
    pub const CHROOT: SyscallNumber = 161;
    pub const SYNC: SyscallNumber = 162;
    pub const ACCT: SyscallNumber = 163;
    pub const SETTIMEOFDAY: SyscallNumber = 164;
    pub const MOUNT: SyscallNumber = 165;
    pub const UMOUNT2: SyscallNumber = 166;
    pub const SWAPON: SyscallNumber = 167;
    pub const SWAPOFF: SyscallNumber = 168;
    pub const REBOOT: SyscallNumber = 169;
    pub const SETHOSTNAME: SyscallNumber = 170;
    pub const SETDOMAINNAME: SyscallNumber = 171;
    pub const IOPL: SyscallNumber = 172;
    pub const IOPERM: SyscallNumber = 173;
    pub const CREATE_MODULE: SyscallNumber = 174;
    pub const INIT_MODULE: SyscallNumber = 175;
    pub const DELETE_MODULE: SyscallNumber = 176;
    pub const GET_KERNEL_SYMS: SyscallNumber = 177;
    pub const QUERY_MODULE: SyscallNumber = 178;
    pub const QUOTACTL: SyscallNumber = 179;
    pub const NFSSERVCTL: SyscallNumber = 180;
    pub const GETPMSG: SyscallNumber = 181;
    pub const PUTPMSG: SyscallNumber = 182;
    pub const AFS_SYSCALL: SyscallNumber = 183;
    pub const TUXCALL: SyscallNumber = 184;
    pub const SECURITY: SyscallNumber = 185;
    pub const GETTID: SyscallNumber = 186;
    pub const READAHEAD: SyscallNumber = 187;
    pub const SETXATTR: SyscallNumber = 188;
    pub const LSETXATTR: SyscallNumber = 189;
    pub const FSETXATTR: SyscallNumber = 190;
    pub const GETXATTR: SyscallNumber = 191;
    pub const LGETXATTR: SyscallNumber = 192;
    pub const FGETXATTR: SyscallNumber = 193;
    pub const LISTXATTR: SyscallNumber = 194;
    pub const LLISTXATTR: SyscallNumber = 195;
    pub const FLISTXATTR: SyscallNumber = 196;
    pub const REMOVEXATTR: SyscallNumber = 197;
    pub const LREMOVEXATTR: SyscallNumber = 198;
    pub const FREMOVEXATTR: SyscallNumber = 199;
    pub const TKILL: SyscallNumber = 200;
    pub const TIME: SyscallNumber = 201;
    pub const FUTEX: SyscallNumber = 202;
    pub const SCHED_SETAFFINITY: SyscallNumber = 203;
    pub const SCHED_GETAFFINITY: SyscallNumber = 204;
    pub const SET_THREAD_AREA: SyscallNumber = 205;
    pub const IO_SETUP: SyscallNumber = 206;
    pub const IO_DESTROY: SyscallNumber = 207;
    pub const IO_GETEVENTS: SyscallNumber = 208;
    pub const IO_SUBMIT: SyscallNumber = 209;
    pub const IO_CANCEL: SyscallNumber = 210;
    pub const GET_THREAD_AREA: SyscallNumber = 211;
    pub const LOOKUP_DCOOKIE: SyscallNumber = 212;
    pub const EPOLL_CREATE: SyscallNumber = 213;
    pub const EPOLL_CTL_OLD: SyscallNumber = 214;
    pub const EPOLL_WAIT_OLD: SyscallNumber = 215;
    pub const REMAP_FILE_PAGES: SyscallNumber = 216;
    pub const GETDENTS64: SyscallNumber = 217;
    pub const SET_TID_ADDRESS: SyscallNumber = 218;
    pub const RESTART_SYSCALL: SyscallNumber = 219;
    pub const SEMTIMEDOP: SyscallNumber = 220;
    pub const FADVISE64: SyscallNumber = 221;
    pub const TIMER_CREATE: SyscallNumber = 222;
    pub const TIMER_SETTIME: SyscallNumber = 223;
    pub const TIMER_GETTIME: SyscallNumber = 224;
    pub const TIMER_GETOVERRUN: SyscallNumber = 225;
    pub const TIMER_DELETE: SyscallNumber = 226;
    pub const CLOCK_SETTIME: SyscallNumber = 227;
    pub const CLOCK_GETTIME: SyscallNumber = 228;
    pub const CLOCK_GETRES: SyscallNumber = 229;
    pub const CLOCK_NANOSLEEP: SyscallNumber = 230;
    pub const EXIT_GROUP: SyscallNumber = 231;
    pub const EPOLL_WAIT: SyscallNumber = 232;
    pub const EPOLL_CTL: SyscallNumber = 233;
    pub const TGKILL: SyscallNumber = 234;
    pub const UTIMES: SyscallNumber = 235;
    pub const VSERVER: SyscallNumber = 236;
    pub const MBIND: SyscallNumber = 237;
    pub const SET_MEMPOLICY: SyscallNumber = 238;
    pub const GET_MEMPOLICY: SyscallNumber = 239;
    pub const MQ_OPEN: SyscallNumber = 240;
    pub const MQ_UNLINK: SyscallNumber = 241;
    pub const MQ_TIMEDSEND: SyscallNumber = 242;
    pub const MQ_TIMEDRECEIVE: SyscallNumber = 243;
    pub const MQ_NOTIFY: SyscallNumber = 244;
    pub const MQ_GETSETATTR: SyscallNumber = 245;
    pub const KEXEC_LOAD: SyscallNumber = 246;
    pub const WAITID: SyscallNumber = 247;
    pub const ADD_KEY: SyscallNumber = 248;
    pub const REQUEST_KEY: SyscallNumber = 249;
    pub const KEYCTL: SyscallNumber = 250;
    pub const IOPRIO_SET: SyscallNumber = 251;
    pub const IOPRIO_GET: SyscallNumber = 252;
    pub const INOTIFY_INIT: SyscallNumber = 253;
    pub const INOTIFY_ADD_WATCH: SyscallNumber = 254;
    pub const INOTIFY_RM_WATCH: SyscallNumber = 255;
    pub const MIGRATE_PAGES: SyscallNumber = 256;
    pub const OPENAT: SyscallNumber = 257;
    pub const MKDIRAT: SyscallNumber = 258;
    pub const MKNODAT: SyscallNumber = 259;
    pub const FCHOWNAT: SyscallNumber = 260;
    pub const FUTIMESAT: SyscallNumber = 261;
    pub const NEWFSTATAT: SyscallNumber = 262;
    pub const UNLINKAT: SyscallNumber = 263;
    pub const RENAMEAT: SyscallNumber = 264;
    pub const LINKAT: SyscallNumber = 265;
    pub const SYMLINKAT: SyscallNumber = 266;
    pub const READLINKAT: SyscallNumber = 267;
    pub const FCHMODAT: SyscallNumber = 268;
    pub const FACCESSAT: SyscallNumber = 269;
    pub const PSELECT6: SyscallNumber = 270;
    pub const PPOLL: SyscallNumber = 271;
    pub const UNSHARE: SyscallNumber = 272;
    pub const SET_ROBUST_LIST: SyscallNumber = 273;
    pub const GET_ROBUST_LIST: SyscallNumber = 274;
    pub const SPLICE: SyscallNumber = 275;
    pub const TEE: SyscallNumber = 276;
    pub const SYNC_FILE_RANGE: SyscallNumber = 277;
    pub const VMSPLICE: SyscallNumber = 278;
    pub const MOVE_PAGES: SyscallNumber = 279;
    pub const UTIMENSAT: SyscallNumber = 280;
    pub const EPOLL_PWAIT: SyscallNumber = 281;
    pub const SIGNALFD: SyscallNumber = 282;
    pub const TIMERFD_CREATE: SyscallNumber = 283;
    pub const EVENTFD: SyscallNumber = 284;
    pub const FALLOCATE: SyscallNumber = 285;
    pub const TIMERFD_SETTIME: SyscallNumber = 286;
    pub const TIMERFD_GETTIME: SyscallNumber = 287;
    pub const ACCEPT4: SyscallNumber = 288;
    pub const SIGNALFD4: SyscallNumber = 289;
    pub const EVENTFD2: SyscallNumber = 290;
    pub const EPOLL_CREATE1: SyscallNumber = 291;
    pub const DUP3: SyscallNumber = 292;
    pub const PIPE2: SyscallNumber = 293;
    pub const INOTIFY_INIT1: SyscallNumber = 294;
    pub const PREADV: SyscallNumber = 295;
    pub const PWRITEV: SyscallNumber = 296;
    pub const RT_TGSIGQUEUEINFO: SyscallNumber = 297;
    pub const PERF_EVENT_OPEN: SyscallNumber = 298;
    pub const RECVMMSG: SyscallNumber = 299;
    pub const FANOTIFY_INIT: SyscallNumber = 300;
    pub const FANOTIFY_MARK: SyscallNumber = 301;
    pub const PRLIMIT64: SyscallNumber = 302;
    pub const NAME_TO_HANDLE_AT: SyscallNumber = 303;
    pub const OPEN_BY_HANDLE_AT: SyscallNumber = 304;
    pub const CLOCK_ADJTIME: SyscallNumber = 305;
    pub const SYNCFS: SyscallNumber = 306;
    pub const SENDMMSG: SyscallNumber = 307;
    pub const SETNS: SyscallNumber = 308;
    pub const GETCPU: SyscallNumber = 309;
    pub const PROCESS_VM_READV: SyscallNumber = 310;
    pub const PROCESS_VM_WRITEV: SyscallNumber = 311;
    pub const KCMP: SyscallNumber = 312;
    pub const FINIT_MODULE: SyscallNumber = 313;
    pub const SCHED_SETATTR: SyscallNumber = 314;
    pub const SCHED_GETATTR: SyscallNumber = 315;
    pub const RENAMEAT2: SyscallNumber = 316;
    pub const SECCOMP: SyscallNumber = 317;
    pub const GETRANDOM: SyscallNumber = 318;
    pub const MEMFD_CREATE: SyscallNumber = 319;
    pub const KEXEC_FILE_LOAD: SyscallNumber = 320;
    pub const BPF: SyscallNumber = 321;
    pub const EXECVEAT: SyscallNumber = 322;
    pub const USERFAULTFD: SyscallNumber = 323;
    pub const MEMBARRIER: SyscallNumber = 324;
    pub const MLOCK2: SyscallNumber = 325;
    pub const COPY_FILE_RANGE: SyscallNumber = 326;
    pub const PREADV2: SyscallNumber = 327;
    pub const PWRITEV2: SyscallNumber = 328;
    pub const PKEY_MPROTECT: SyscallNumber = 329;
    pub const PKEY_ALLOC: SyscallNumber = 330;
    pub const PKEY_FREE: SyscallNumber = 331;
    pub const STATX: SyscallNumber = 332;
    pub const IO_PGETEVENTS: SyscallNumber = 333;
    pub const RSEQ: SyscallNumber = 334;
    pub const PIDFD_SEND_SIGNAL: SyscallNumber = 424;
    pub const IO_URING_SETUP: SyscallNumber = 425;
    pub const IO_URING_ENTER: SyscallNumber = 426;
    pub const IO_URING_REGISTER: SyscallNumber = 427;
    pub const OPEN_TREE: SyscallNumber = 428;
    pub const MOVE_MOUNT: SyscallNumber = 429;
    pub const FSOPEN: SyscallNumber = 430;
    pub const FSCONFIG: SyscallNumber = 431;
    pub const FSMOUNT: SyscallNumber = 432;
    pub const FSPICK: SyscallNumber = 433;
    pub const PIDFD_OPEN: SyscallNumber = 434;
    pub const CLONE3: SyscallNumber = 435;
    pub const CLOSE_RANGE: SyscallNumber = 436;
    pub const OPENAT2: SyscallNumber = 437;
    pub const PIDFD_GETFD: SyscallNumber = 438;
    pub const FACCESSAT2: SyscallNumber = 439;
    pub const PROCESS_MADVISE: SyscallNumber = 440;
    pub const EPOLL_PWAIT2: SyscallNumber = 441;
    pub const MOUNT_SETATTR: SyscallNumber = 442;
    pub const LANDLOCK_CREATE_RULESET: SyscallNumber = 444;
    pub const LANDLOCK_ADD_RULE: SyscallNumber = 445;
    pub const LANDLOCK_RESTRICT_SELF: SyscallNumber = 446;
    pub const MEMFD_SECRET: SyscallNumber = 447;
    pub const PROCESS_MRELEASE: SyscallNumber = 448;
    pub const FUTEX_WAITV: SyscallNumber = 449;
    pub const SET_MEMPOLICY_HOME_NODE: SyscallNumber = 450;
    pub const CACHESTAT: SyscallNumber = 451;
    pub const FCHMODAT2: SyscallNumber = 452;
}
