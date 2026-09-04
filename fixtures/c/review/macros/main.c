#include <stdio.h>
#include "callbacks.h"

HANDLER(default_handler) {
    int clamped = CLAMP(code, 0, MAX_ITEMS);
    printf("code=%d\n", clamped);
    return clamped;
}

int dispatch(event_handler_t handler, void *ctx, int code) {
    return handler(ctx, code);
}

static notify_fn active_notify = 0;

void register_notify(notify_fn fn) {
    active_notify = fn;
}

int main(void) {
    event_handler_t h = default_handler;
    int result = dispatch(h, 0, 200);
    printf("%d\n", result);
    return 0;
}
