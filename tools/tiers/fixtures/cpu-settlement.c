#define _GNU_SOURCE
/* Model a continuous reap stream: tools/tiers/design.md, 7.4. */
#include <errno.h>
#include <fcntl.h>
#include <signal.h>
#include <stdarg.h>
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
extern int __real_open(const char *, int, ...);
extern ssize_t __real_read(int, void *, size_t);
extern off_t __real_lseek(int, off_t, int);
extern int __real_kill(pid_t, int);
extern ssize_t __real_write(int, const void *, size_t);
static bool reaped;
static unsigned stream;
static struct timespec origin;
static int children_fd = -1;
static size_t child_offset;

int __wrap_open(const char *path, int flags, ...) {
    mode_t mode = 0;
    if (flags & O_CREAT) {
        va_list args;
        va_start(args, flags);
        mode = va_arg(args, mode_t);
        va_end(args);
    }
    int fd = __real_open(path, flags, mode);
    if (strstr(path, "/children")) children_fd = fd;
    return fd;
}
off_t __wrap_lseek(int fd, off_t offset, int origin) {
    if (!reaped || fd != children_fd) return __real_lseek(fd, offset, origin);
    child_offset = (size_t)offset;
    return offset;
}
ssize_t __wrap_read(int fd, void *data, size_t size) {
    if (!reaped || fd != children_fd) return __real_read(fd, data, size);
    static const char child[] = "2000000\n";
    size_t remaining = sizeof child - 1 - child_offset;
    /* Split the PID across reads to check the retained descriptor's parser. */
    if (remaining > 2) remaining = 2;
    if (remaining > size) remaining = size;
    memcpy(data, child + child_offset, remaining);
    child_offset += remaining;
    return (ssize_t)remaining;
}

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
int __wrap_kill(pid_t child, int sig) {
    if (getenv("TIERS_REFUSE_KILL")) { errno = EPERM; return -1; }
    if (child != 2000000) return __real_kill(child, sig);
    const char *marker = getenv("TIERS_CLEANUP_MARKER");
    FILE *out = marker ? fopen(marker, "a") : NULL;
    if (!out) abort();
    fprintf(out, "%d\n", sig);
    fclose(out);
    return 0;
}
