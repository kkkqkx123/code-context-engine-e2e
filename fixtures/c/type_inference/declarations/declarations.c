#include <stdio.h>
#include <stdlib.h>

typedef struct {
    int x;
    int y;
} Point;

typedef unsigned long size_alias;

enum Color {
    RED,
    GREEN,
    BLUE
};

struct Counter {
    int value;
};

int add(int a, int b) {
    return a + b;
}

double distance(Point a, Point b) {
    int dx = a.x - b.x;
    int dy = a.y - b.y;
    return dx + dy;
}

size_alias count_items(const int *items, size_alias n) {
    size_alias total = 0;
    for (size_alias i = 0; i < n; i++) {
        total += items[i];
    }
    return total;
}

int global_total = 0;

int main(void) {
    Point origin = {0, 0};
    int count = 42;
    struct Counter counter;
    counter.value = count;
    enum Color favorite = GREEN;
    size_alias n = count_items(&count, 1);
    global_total = add(count, (int)n);
    printf("%d\n", global_total);
    return 0;
}
