#define _GNU_SOURCE
/* Force reuse of a published basename: tools/tiers/design.md, 7.4. */
#include <fcntl.h>
#include <stdbool.h>
#include <string.h>
#include <unistd.h>

extern int __real_mkstemps(char *, int);

int __wrap_mkstemps(char *path, int suffix) {
    static bool first = true;
    if (!first) return __real_mkstemps(path, suffix);
    first = false;
    memcpy(path + strlen(path) - (size_t)suffix - 6, "AAAAAA", 6);
    return open(path, O_RDWR | O_CREAT | O_EXCL, 0600);
}
