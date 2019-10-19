#include <fnmatch.h>
#include <stdio.h>

int main(int argc, char **argv) {
    const char *pattern;
    const char *string;
    int flags = 0;
    int rc = 0;

    if (argc < 3) {
        fprintf(stderr, "Usage: fnmatch PATTERN STRING\n");
        return 1;
    }

    pattern = argv[1];
    string = argv[2];
    printf("pattern: %s\n", pattern);
    printf("string: %s\n", string);

    rc = fnmatch(pattern, string, flags);
    printf("fnmatch(): %d\n", rc);

    return rc == 0 ? 0 : 1;
}
