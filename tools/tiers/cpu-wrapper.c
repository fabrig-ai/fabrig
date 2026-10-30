#define _GNU_SOURCE
/* Per-attempt Linux CPU accounting: tools/tiers/design.md, 7. */
#include <errno.h>
#include <fcntl.h>
#include <math.h>
#include <signal.h>
#include <spawn.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/prctl.h>
#include <sys/resource.h>
#include <sys/stat.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

extern char **environ;
static pid_t payload_pid = -1;
static bool payload_group;
_Noreturn static void collector_failure(const char *message);

static double seconds(struct timeval value) {
    return (double)value.tv_sec + (double)value.tv_usec / 1000000.0;
}
static double now(void) {
    struct timespec value;
    if (clock_gettime(CLOCK_MONOTONIC, &value)) collector_failure("cpu-wrapper: clock");
    return (double)value.tv_sec + (double)value.tv_nsec / 1000000000.0;
}
static const char *required(const char *key) {
    const char *value = getenv(key);
    if (!value || !*value) { fprintf(stderr, "cpu-wrapper: missing %s\n", key); exit(125); }
    return value;
}
static char *record_data, *temporary_path;
static int initial_fd = -1;
static bool suite_record, starved;
static const char *profile, *binary_id, *test_name;
static uint64_t attempt_number;
static char *append(char *out, const char *value) {
    size_t length = strlen(value);
    memcpy(out, value, length);
    return out + length;
}
static char *quoted(char *out, const char *value) {
    static const char hex[] = "0123456789abcdef";
    *out++ = '"';
    for (const unsigned char *p = (const unsigned char *)value; *p; ++p) {
        if (*p == '"' || *p == '\\') { *out++ = '\\'; *out++ = (char)*p; }
        else if (*p < 32) {
            out = append(out, "\\u00"); *out++ = hex[*p / 16]; *out++ = hex[*p % 16];
        } else *out++ = (char)*p;
    }
    *out++ = '"';
    return out;
}
static char *integer(char *out, uint64_t value) {
    char digits[32];
    size_t length = 0;
    do { digits[length++] = (char)('0' + value % 10); value /= 10; } while (value);
    while (length) *out++ = digits[--length];
    return out;
}
static char *decimal(char *out, double value, uint64_t scale) {
    uint64_t ticks = (uint64_t)(value * (double)scale + 0.5);
    out = integer(out, ticks / scale);
    *out++ = '.';
    uint64_t fraction = ticks % scale;
    for (uint64_t place = scale / 10; place; place /= 10) {
        *out++ = (char)('0' + fraction / place);
        fraction %= place;
    }
    return out;
}
static void record(const char *path, double start, double user, double system,
                   int status, bool complete, unsigned leaked, bool pending, bool settling) {
    /* Reuse the identity buffer and write fixed decimal seconds without stdio. */
    char *out = append(record_data, "{\"profile\":");
    out = quoted(out, profile);
    out = append(out, ",\"binary_id\":"); out = quoted(out, binary_id);
    out = append(out, ",\"test_name\":"); out = quoted(out, test_name);
    out = append(out, ",\"attempt\":"); out = integer(out, attempt_number);
    out = append(out, ",\"user_s\":"); out = decimal(out, user, 1000000);
    out = append(out, ",\"system_s\":"); out = decimal(out, system, 1000000);
    out = append(out, ",\"wall_s\":"); out = decimal(out, now() - start, 1000000000);
    out = append(out, ",\"start_s\":"); out = decimal(out, start, 1000000000);
    struct rusage own;
    if (getrusage(RUSAGE_SELF, &own)) collector_failure("cpu-wrapper: own CPU");
    out = append(out, ",\"wrapper_cpu_s\":");
    out = decimal(out, seconds(own.ru_utime) + seconds(own.ru_stime), 1000000);
    out = append(out, ",\"complete\":"); out = append(out, complete ? "true" : "false");
    out = append(out, ",\"leaked\":"); out = integer(out, leaked);
    out = append(out, ",\"pending\":"); out = append(out, pending ? "true" : "false");
    out = append(out, ",\"settling\":"); out = append(out, settling ? "true" : "false");
    out = append(out, ",\"complete_scope\":\"waited-and-reaped-descendants\",\"interrupted\":0,\"exit_code\":");
    out = !pending && WIFEXITED(status) ? integer(out, WEXITSTATUS(status)) : append(out, "null");
    out = append(out, ",\"signal\":");
    out = !pending && WIFSIGNALED(status) ? integer(out, WTERMSIG(status)) : append(out, "null");
    if (suite_record) {
        out = append(out, ",\"starved\":"); out = append(out, starved ? "true" : "false");
    }
    out = append(out, "}\n");
    size_t length = (size_t)(out - record_data);
    int fd = initial_fd;
    bool initial = fd >= 0;
    initial_fd = -1;
    if (fd < 0) fd = open(temporary_path, O_WRONLY | O_CREAT | O_EXCL | O_CLOEXEC, 0600);
    if (fd < 0 || write(fd, record_data, length) != (ssize_t)length || close(fd)) {
        collector_failure("cpu-wrapper: write record");
    }
    /* A named suite must not replace another invocation's record. */
    int published = suite_record && initial
        ? (link(temporary_path, path) || unlink(temporary_path))
        : rename(temporary_path, path);
    if (published) {
        collector_failure("cpu-wrapper: publish record");
    }
}
/* Only our direct children: adoption brings a daemon here even after setsid. */
static bool signal_children(int sig, double deadline) {
    char path[96];
    snprintf(path, sizeof path, "/proc/self/task/%ld/children", (long)getpid());
    FILE *children = fopen(path, "r");
    if (!children) return false;
    long child;
    while (fscanf(children, "%ld", &child) == 1) {
        if (deadline && now() >= deadline) { fclose(children); errno = ETIMEDOUT; return false; }
        if (kill((pid_t)child, sig) && errno != ESRCH) {
            int error = errno;
            fclose(children); errno = error; return false;
        }
    }
    bool success = !ferror(children);
    int error = errno;
    fclose(children);
    if (!success) errno = error ? error : EIO;
    return success;
}
static bool kill_children(double deadline) {
    bool success = signal_children(SIGKILL, deadline);
    if (!success && errno != ETIMEDOUT) collector_failure("cpu-wrapper: child cleanup");
    return success;
}
static void cleanup_after_error(void) {
    if (payload_pid > 0) {
        /* Error cleanup uses neither the collector's clock nor record writer.
           Nextest owns the untimed group; signal only our children there. */
        if (payload_group) kill(-payload_pid, SIGTERM);
        for (unsigned tick = 0; tick < 2000; ++tick) {
            if (tick == 1000 && payload_group) kill(-payload_pid, SIGKILL);
            signal_children(tick < 1000 ? SIGTERM : SIGKILL, 0);
            /* A reap stream must not prevent escalation or bound checking. */
            for (unsigned count = 0; count < 64; ++count) {
                int status;
                struct rusage usage;
                pid_t child = wait4(-1, &status, WNOHANG | __WALL, &usage);
                if (child < 0 && errno == ECHILD) return;
                if (child <= 0) break;
            }
            struct timespec pause = { .tv_nsec = 1000000 };
            nanosleep(&pause, NULL);
        }
    }
}
_Noreturn static void collector_failure(const char *message) {
    int error = errno;
    cleanup_after_error();
    /* A broken diagnostic pipe must not interrupt cleanup or change exit 125. */
    sigset_t blocked;
    sigemptyset(&blocked); sigaddset(&blocked, SIGPIPE);
    sigprocmask(SIG_BLOCK, &blocked, NULL);
    errno = error;
    perror(message);
    exit(125);
}
int main(int argc, char **argv) {
    umask(077);
    if (argc < 3) { fputs("usage: cpu-wrapper DIR TEST [ARGS...]\n", stderr); return 125; }
    int command = 2;
    double wall_hard = 0;
    suite_record = strcmp(argv[1], "--suite") == 0;
    if (suite_record) {
        command = argc > 4 && strcmp(argv[4], "--wall-hard") == 0 ? 7 : 5;
        if (argc <= command || strcmp(argv[command - 1], "--") || !*argv[2] || !*argv[3]) {
            fputs("usage: cpu-wrapper --suite RECORD ID [--wall-hard SECONDS] -- COMMAND [ARGS...]\n", stderr);
            return 125;
        }
        if (command == 7) {
            char *end;
            wall_hard = strtod(argv[5], &end);
            if (*end || !isfinite(wall_hard) || wall_hard <= 0) {
                fputs("cpu-wrapper: wall-hard must be finite positive seconds\n", stderr);
                return 125;
            }
        }
        profile = "tools";
        binary_id = argv[3];
        test_name = "suite";
        attempt_number = 1;
    } else {
        /* Validate identity before starting the test; never silently omit accounting. */
        profile = required("NEXTEST_PROFILE");
        binary_id = required("NEXTEST_BINARY_ID");
        test_name = required("NEXTEST_TEST_NAME");
        const char *attempt = required("NEXTEST_ATTEMPT");
        char *end;
        long number = strtol(attempt, &end, 10);
        if (*end || number < 1) { fputs("cpu-wrapper: invalid attempt\n", stderr); return 125; }
        attempt_number = (uint64_t)number;
    }
    size_t names = strlen(profile) + strlen(binary_id) + strlen(test_name);
    if (names > (SIZE_MAX - 1024) / 6) return 125;
    record_data = malloc(1024 + 6 * names);
    if (!record_data) { perror("cpu-wrapper: record buffer"); return 125; }
    if (prctl(PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0)) { perror("cpu-wrapper: subreaper"); return 125; }
    struct sigaction action = { .sa_handler = SIG_IGN };
    sigemptyset(&action.sa_mask);
    const int signals[] = { SIGTERM, SIGINT, SIGHUP, SIGQUIT };
    sigset_t defaults;
    sigemptyset(&defaults);
    for (size_t i = 0; i < sizeof signals / sizeof signals[0]; ++i) {
        if (sigaction(signals[i], &action, NULL)) { perror("cpu-wrapper: sigaction"); return 125; }
        sigaddset(&defaults, signals[i]);
    }
    const char *destination = suite_record ? argv[2] : argv[1];
    char *path = malloc(strlen(destination) + sizeof "/cpu-XXXXXX.tmp");
    if (!path) return 125;
    strcpy(path, destination);
    strcat(path, suite_record ? ".XXXXXX.tmp" : "/cpu-XXXXXX.tmp");
    int fd = mkstemps(path, 4);
    if (fd < 0) { perror("cpu-wrapper: attempt file"); return 125; }
    initial_fd = fd;
    temporary_path = strdup(path);
    if (!temporary_path) return 125;
    if (suite_record) strcpy(path, destination);
    else path[strlen(path) - 4] = '\0';
    double start = now();
    /* A forced SIGKILL leaves this explicit incomplete record, never invented CPU. */
    record(path, start, 0, 0, 0, false, 0, true, false);
    pid_t test;
    /* Nextest signals our group. Ignore those signals in the supervisor, and
       restore their defaults only in the child: no forwarding or event queue. */
    posix_spawnattr_t spawn;
    int launch = posix_spawnattr_init(&spawn);
    if (launch) { errno = launch; perror("cpu-wrapper: spawn attributes"); return 125; }
    launch = posix_spawnattr_setsigdefault(&spawn, &defaults);
    if (!launch && wall_hard) launch = posix_spawnattr_setpgroup(&spawn, 0);
    if (!launch) launch = posix_spawnattr_setflags(&spawn, POSIX_SPAWN_SETSIGDEF
                                                 | (wall_hard ? POSIX_SPAWN_SETPGROUP : 0));
    if (!launch) launch = posix_spawnp(&test, argv[command], NULL, &spawn, argv + command, environ);
    if (!launch) {
        payload_pid = test;
        payload_group = wall_hard != 0;
    }
    int destroyed = posix_spawnattr_destroy(&spawn);
    if (destroyed) { errno = destroyed; collector_failure("cpu-wrapper: spawn attributes cleanup"); }
    if (launch) {
        errno = launch; perror("cpu-wrapper: exec");
        record(path, start, 0, 0, 127 << 8, true, 0, false, false);
        free(path); return 127;
    }
    double user = 0, system = 0, settled = 0, terminate_at = 0;
    int status = 0;
    bool exited = false, complete = false, killing = false, cleanup_ok = true;
    unsigned leaked = 0;
    for (;;) {
        if (wall_hard && !starved && now() - start >= wall_hard) {
            starved = true;
            terminate_at = now() + 1.0;
            if (kill(-test, SIGTERM) && errno != ESRCH) collector_failure("cpu-wrapper: terminate payload group");
            record(path, start, user, system, status, false, leaked, !exited, exited);
        }
        if (starved) {
            double current = now();
            if (current >= terminate_at) {
                if (!killing && kill(-test, SIGKILL) && errno != ESRCH) collector_failure("cpu-wrapper: kill payload group");
                killing = true;
                if (!kill_children(terminate_at + 1.0)) cleanup_ok = false;
            }
            if (current >= terminate_at + 1.0) break;
        } else if (exited) {
            double elapsed = now() - settled;
            if (elapsed >= 0.1) {
                killing = true;
                if (!kill_children(settled + 1.0)) cleanup_ok = false;
            }
            if (elapsed >= 1.0) break;
        }
        struct rusage usage;
        int child_status;
        pid_t child = wait4(-1, &child_status, (exited || wall_hard ? WNOHANG : 0) | __WALL, &usage);
        if (child > 0) {
            user += seconds(usage.ru_utime); system += seconds(usage.ru_stime);
            if (child == test) {
                status = child_status; exited = true; settled = now();
                record(path, start, user, system, status, false, leaked, false, true);
            }
            else if (killing && WIFSIGNALED(child_status) && WTERMSIG(child_status) == SIGKILL) ++leaked;
            continue;
        }
        if (child < 0) {
            if (errno == EINTR) continue;
            if (errno != ECHILD) collector_failure("cpu-wrapper: wait4");
            complete = errno == ECHILD && exited && cleanup_ok;
            break;
        }
        struct timespec pause = { .tv_nsec = 1000000 };
        if (nanosleep(&pause, NULL) && errno != EINTR) collector_failure("cpu-wrapper: sleep");
    }
    record(path, start, user, system, status, complete, leaked, !exited, false);
    free(path); free(temporary_path); free(record_data);
    if (!exited) return 125;
    if (WIFSIGNALED(status)) {
        int sig = WTERMSIG(status);
        signal(sig, SIG_DFL);
        sigset_t unblocked;
        sigemptyset(&unblocked); sigaddset(&unblocked, sig);
        sigprocmask(SIG_UNBLOCK, &unblocked, NULL);
        raise(sig);
        return 125;
    }
    return WEXITSTATUS(status);
}
