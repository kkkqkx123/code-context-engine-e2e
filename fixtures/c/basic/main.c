#include <stdio.h>
#include "helper.h"

int main(void) {
    Point origin = {0, 0};
    Point target = {3, 4};
    int total = add(origin.x, target.x);
    double dist = distance(origin, target);
    printf("%d %f\n", total, dist);
    return 0;
}
