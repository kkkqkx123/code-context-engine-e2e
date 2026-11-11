#include <stdio.h>
#include "helper.h"

#ifdef WITH_EXTRA
#include "extra.h"
#endif

#define CLAMP(v) ((v) > 0 ? (v) : 0)

int apply(int (*fp)(int), int v) {
    return fp(v);
}

int main(void) {
    printf("hi\n");
    int capped = CLAMP(MAX(1, 2));
    return apply(main, capped);
}
