#define _GNU_SOURCE
/* Model a continuous reap stream: tools/tiers/design.md, 7.4. */
#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

extern pid_t __real_wait4(pid_t, int *, int, struct rusage *);
extern int __real_clock_gettime(clockid_t, struct timespec *);
extern FILE *__real_fopen(const char *, const char *);
extern int __real_kill(pid_t, int);
extern ssize_t __real_write(int, const void *, size_t);
static bool reaped;
static unsigned stream;
static struct timespec origin;

ssize_t __wrap_write(int fd, const void *data, size_t size) {
    const char *trace = getenv("TIERS_RECORD_TRACE");
    if (trace) {
        int output = open(trace, O_WRONLY | O_APPEND | O_CREAT, 0600);
        if (output < 0 || __real_write(output, data, size) != (ssize_t)size || close(output)) abort();
    }
    return __real_write(fd, data, size);
}

pid_t __wrap_wait4(pid_t child, int *status, int options, struct rusage *usage) {
    if (!reaped) {
        pid_t result = __real_wait4(child, status, options, usage);
        if (result > 0) {
            reaped = true;
            __real_clock_gettime(CLOCK_MONOTONIC, &origin);
        }
        return result;
    }
    if (stream++ < 20) {
        *status = 0;
        memset(usage, 0, sizeof *usage);
        return 2000000;
    }
    errno = ECHILD;
    return -1;
}
int __wrap_clock_gettime(clockid_t clock, struct timespec *value) {
    if (!reaped || clock != CLOCK_MONOTONIC) return __real_clock_gettime(clock, value);
    *value = origin;
    value->tv_sec += stream / 10;
    value->tv_nsec += (stream % 10) * 100000000L;
    if (value->tv_nsec >= 1000000000L) { ++value->tv_sec; value->tv_nsec -= 1000000000L; }
    return 0;
}
FILE *__wrap_fopen(const char *path, const char *mode) {
    if (reaped && strstr(path, "/children")) {
        static char child[] = "2000000\n";
        return fmemopen(child, strlen(child), "r");
    }
    return __real_fopen(path, mode);
}
int __wrap_kill(pid_t child, int sig) {
    if (child != 2000000) return __real_kill(child, sig);
    const char *marker = getenv("TIERS_CLEANUP_MARKER");
    FILE *out = marker ? __real_fopen(marker, "a") : NULL;
    if (!out) abort();
    fprintf(out, "%d\n", sig);
    fclose(out);
    return 0;
}
