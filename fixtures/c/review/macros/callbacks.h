#ifndef CALLBACKS_H
#define CALLBACKS_H

#define MAX_ITEMS 128
#define CLAMP(v, lo, hi) ((v) < (lo) ? (lo) : ((v) > (hi) ? (hi) : (v)))
#define HANDLER(name) int name(void *ctx, int code)

typedef int (*event_handler_t)(void *ctx, int code);
typedef void (*notify_fn)(const char *message);

HANDLER(default_handler);
int dispatch(event_handler_t handler, void *ctx, int code);
void register_notify(notify_fn fn);

#endif
