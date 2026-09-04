#include "helper.h"

int add(int a, int b) {
    return a + b;
}

double distance(Point a, Point b) {
    int dx = a.x - b.x;
    int dy = a.y - b.y;
    return dx + dy;
}
