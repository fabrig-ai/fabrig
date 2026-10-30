#define _GNU_SOURCE
/* Per-attempt Linux CPU accounting: tools/tiers/design.md, 7. */
#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <spawn.h>
#include <stdbool.h>
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
static volatile sig_atomic_t interrupted;
static void remember(int sig) { interrupted = sig; }
static double seconds(struct timeval value) {
    return (double)value.tv_sec + (double)value.tv_usec / 1000000.0;
}
static double now(void) {
    struct timespec value;
    if (clock_gettime(CLOCK_MONOTONIC, &value)) { perror("cpu-wrapper: clock"); exit(125); }
    return (double)value.tv_sec + (double)value.tv_nsec / 1000000000.0;
}
static const char *required(const char *key) {
    const char *value = getenv(key);
    if (!value || !*value) { fprintf(stderr, "cpu-wrapper: missing %s\n", key); exit(125); }
    return value;
}
static void quoted(FILE *out, const char *value) {
    fputc('"', out);
    for (const unsigned char *p = (const unsigned char *)value; *p; ++p) {
        if (*p == '"' || *p == '\\') { fputc('\\', out); fputc(*p, out); }
        else if (*p < 32) { fprintf(out, "\\u%04x", *p); }
        else { fputc(*p, out); }
    }
    fputc('"', out);
}
static void record(const char *path, double start, double user, double system,
                   int status, bool complete, unsigned leaked, bool pending) {
    char *data = NULL, *temporary = NULL;
    size_t length = 0;
    FILE *out = open_memstream(&data, &length);
    if (!out) { perror("cpu-wrapper: record buffer"); exit(125); }
    fputs("{\"profile\":", out); quoted(out, required("NEXTEST_PROFILE"));
    fputs(",\"binary_id\":", out); quoted(out, required("NEXTEST_BINARY_ID"));
    fputs(",\"test_name\":", out); quoted(out, required("NEXTEST_TEST_NAME"));
    fputs(",\"attempt\":", out); fputs(required("NEXTEST_ATTEMPT"), out);
    struct rusage own;
    if (getrusage(RUSAGE_SELF, &own)) { perror("cpu-wrapper: own CPU"); exit(125); }
    fprintf(out, ",\"user_s\":%.6f,\"system_s\":%.6f,\"wall_s\":%.9f,\"start_s\":%.9f,"
                 "\"wrapper_cpu_s\":%.6f,\"complete\":%s,\"leaked\":%u,\"pending\":%s,"
                 "\"interrupted\":%d,\"exit_code\":",
            user, system, now() - start, start, seconds(own.ru_utime) + seconds(own.ru_stime),
            complete ? "true" : "false", leaked, pending ? "true" : "false", (int)interrupted);
    if (!pending && WIFEXITED(status)) fprintf(out, "%d", WEXITSTATUS(status));
    else fputs("null", out);
    fputs(",\"signal\":", out);
    if (!pending && WIFSIGNALED(status)) fprintf(out, "%d", WTERMSIG(status));
    else fputs("null", out);
    fputs("}\n", out);
    if (fclose(out) || asprintf(&temporary, "%s.tmp", path) < 0) {
        perror("cpu-wrapper: format record"); exit(125);
    }
    int fd = open(temporary, O_WRONLY | O_CREAT | O_EXCL | O_CLOEXEC, 0600);
    if (fd < 0 || write(fd, data, length) != (ssize_t)length || close(fd) || rename(temporary, path)) {
        perror("cpu-wrapper: publish record"); exit(125);
    }
    free(temporary); free(data);
}
/* Only our direct children: adoption brings a daemon here even after setsid. */
static bool kill_children(void) {
    char path[96];
    snprintf(path, sizeof path, "/proc/self/task/%ld/children", (long)getpid());
    FILE *children = fopen(path, "r");
    if (!children) return false;
    long child;
    while (fscanf(children, "%ld", &child) == 1) {
        if (kill((pid_t)child, SIGKILL) && errno != ESRCH) { fclose(children); return false; }
    }
    bool success = !ferror(children);
    fclose(children);
    return success;
}
int main(int argc, char **argv) {
    umask(077);
    if (argc < 3) { fputs("usage: cpu-wrapper DIR TEST [ARGS...]\n", stderr); return 125; }
    /* Validate identity before starting the test; never silently omit accounting. */
    required("NEXTEST_PROFILE"); required("NEXTEST_BINARY_ID"); required("NEXTEST_TEST_NAME");
    const char *attempt = required("NEXTEST_ATTEMPT");
    char *end;
    long number = strtol(attempt, &end, 10);
    if (*end || number < 1) { fputs("cpu-wrapper: invalid attempt\n", stderr); return 125; }
    if (prctl(PR_SET_CHILD_SUBREAPER, 1, 0, 0, 0)) { perror("cpu-wrapper: subreaper"); return 125; }
    struct sigaction action = { .sa_handler = remember };
    sigemptyset(&action.sa_mask);
    const int signals[] = { SIGTERM, SIGINT, SIGHUP, SIGQUIT };
    for (size_t i = 0; i < sizeof signals / sizeof signals[0]; ++i)
        if (sigaction(signals[i], &action, NULL)) { perror("cpu-wrapper: sigaction"); return 125; }
    char *path = NULL;
    if (asprintf(&path, "%s/cpu-XXXXXX.tmp", argv[1]) < 0) return 125;
    int fd = mkstemps(path, 4);
    if (fd < 0) { perror("cpu-wrapper: attempt file"); return 125; }
    close(fd);
    unlink(path);
    path[strlen(path) - 4] = '\0';
    double start = now();
    /* A forced SIGKILL leaves this explicit incomplete record, never invented CPU. */
    record(path, start, 0, 0, 0, false, 0, true);
    pid_t test;
    int launch = posix_spawnp(&test, argv[2], NULL, NULL, argv + 2, environ);
    if (launch) {
        errno = launch; perror("cpu-wrapper: exec");
        record(path, start, 0, 0, 127 << 8, true, 0, false);
        free(path); return 127;
    }
    double user = 0, system = 0, settled = 0;
    int status = 0;
    bool exited = false, complete = false, killing = false, cleanup_ok = true;
    unsigned leaked = 0;
    int forwarded = 0;
    for (;;) {
        if (!exited && interrupted && forwarded != interrupted) {
            kill(test, interrupted);
            forwarded = interrupted;
        }
        struct rusage usage;
        int child_status;
        pid_t child = wait4(-1, &child_status, (exited ? WNOHANG : 0) | __WALL, &usage);
        if (child > 0) {
            user += seconds(usage.ru_utime); system += seconds(usage.ru_stime);
            if (child == test) { status = child_status; exited = true; settled = now(); }
            else if (killing && WIFSIGNALED(child_status) && WTERMSIG(child_status) == SIGKILL) ++leaked;
            continue;
        }
        if (child < 0) {
            if (errno == EINTR) continue;
            complete = errno == ECHILD && exited && cleanup_ok;
            break;
        }
        double elapsed = now() - settled;
        if (elapsed >= 0.1) {
            killing = true;
            if (!kill_children()) cleanup_ok = false;
        }
        if (elapsed >= 1.0) break;
        struct timespec pause = { .tv_nsec = 1000000 };
        nanosleep(&pause, NULL);
    }
    record(path, start, user, system, status, complete, leaked, false);
    free(path);
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
