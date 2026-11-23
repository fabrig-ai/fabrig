/* Count one nextest test process (tools/tiers/design.md, 11.9). */
#define _GNU_SOURCE
#include <errno.h>
#include <fcntl.h>
#include <inttypes.h>
#include <linux/perf_event.h>
#include <signal.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/syscall.h>
#include <sys/wait.h>
#include <unistd.h>

static _Noreturn void exec_test(char **args)
{
    execvp(args[0], args);
    int error = errno;
    perror(args[0]);
    _exit(error == ENOENT ? 127 : 126);
}

static int open_counter(pid_t child)
{
    struct perf_event_attr attr = {
        .size = PERF_ATTR_SIZE_VER0,
        .type = PERF_TYPE_HARDWARE,
        .config = PERF_COUNT_HW_INSTRUCTIONS,
        .read_format = PERF_FORMAT_TOTAL_TIME_ENABLED | PERF_FORMAT_TOTAL_TIME_RUNNING,
        .disabled = 1,
        .inherit = 1,
        .exclude_kernel = 1,
        .exclude_hv = 1,
        .enable_on_exec = 1,
    };
    return (int)syscall(SYS_perf_event_open, &attr, child, -1, -1, PERF_FLAG_FD_CLOEXEC);
}

static void json_string(FILE *stream, const char *value)
{
    fputc('"', stream);
    for (const unsigned char *byte = (const unsigned char *)value; *byte; ++byte) {
        if (*byte == '"' || *byte == '\\') {
            fputc('\\', stream);
            fputc(*byte, stream);
        } else if (*byte < 0x20) {
            fprintf(stream, "\\u%04x", (unsigned int)*byte);
        } else {
            fputc(*byte, stream);
        }
    }
    fputc('"', stream);
}

static void append_count(const char *path, const char *binary, const char *test, uint64_t count)
{
    char *line = NULL;
    size_t length = 0;
    FILE *stream = open_memstream(&line, &length);
    if (!stream) return;
    fputs("{\"binary_id\":", stream);
    json_string(stream, binary);
    fputs(",\"test\":", stream);
    json_string(stream, test);
    fprintf(stream, ",\"instructions\":%" PRIu64 "}\n", count);
    int failed = ferror(stream);
    if (fclose(stream) == 0 && !failed) {
        int fd = open(path, O_WRONLY | O_APPEND | O_CLOEXEC);
        if (fd >= 0) {
            ssize_t written = write(fd, line, length);
            (void)written;
            close(fd);
        }
    }
    free(line);
}

int main(int argc, char **argv)
{
    if (argc < 2) return 126;
    const char *path = getenv("TIERS_COUNTS_OUT");
    const char *phase = getenv("NEXTEST_TEST_PHASE");
    const char *binary = getenv("NEXTEST_BINARY_ID");
    const char *test = getenv("NEXTEST_TEST_NAME");
    if (!path || !*path || !phase || strcmp(phase, "run") || !binary || !test)
        exec_test(argv + 1);

    int start[2];
    if (pipe2(start, O_CLOEXEC) < 0) exec_test(argv + 1);
    pid_t child = fork();
    if (child < 0) {
        close(start[0]);
        close(start[1]);
        exec_test(argv + 1);
    }
    if (child == 0) {
        close(start[1]);
        /* The parent closes its end after opening the counter. */
        char byte;
        while (read(start[0], &byte, 1) < 0 && errno == EINTR) {}
        close(start[0]);
        exec_test(argv + 1);
    }

    close(start[0]);
    int counter = open_counter(child);
    close(start[1]);
    int status;
    pid_t waited;
    do {
        waited = waitpid(child, &status, 0);
    } while (waited < 0 && errno == EINTR);

    if (counter >= 0) {
        uint64_t values[3]; /* Count, time enabled, time running. */
        ssize_t size = read(counter, values, sizeof(values));
        close(counter);
        if (waited == child && size == sizeof(values) && values[1] == values[2])
            append_count(path, binary, test, values[0]);
    }

    if (waited != child) return 126;
    if (WIFSIGNALED(status)) {
        int sig = WTERMSIG(status);
        sigset_t mask;
        sigemptyset(&mask);
        sigaddset(&mask, sig);
        signal(sig, SIG_DFL);
        sigprocmask(SIG_UNBLOCK, &mask, NULL);
        raise(sig);
        return 128 + sig;
    }
    return WEXITSTATUS(status);
}
