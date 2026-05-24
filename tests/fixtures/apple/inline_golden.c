// Fixture for the atos differential tests.
//
// `leaf_inline` and `mid_inline` are force-inlined into `outer`, so at -O2 the
// compiler emits DW_TAG_inlined_subroutine DIEs. Symbolizing an address inside
// `outer` must therefore expand into the inline call stack
// (leaf_inline -> mid_inline -> outer), which is what `atos -i` reports.

#include <stdio.h>

__attribute__((always_inline)) inline int leaf_inline(int x) {
    return x * x + 7; // leaf body
}

__attribute__((always_inline)) inline int mid_inline(int x) {
    return leaf_inline(x) + leaf_inline(x + 1); // calls the leaf twice
}

__attribute__((noinline)) int outer(int x) {
    return mid_inline(x) * 3; // mid + leaf inlined into here
}

int main(int argc, char **argv) {
    printf("%d\n", outer(argc));
    return 0;
}
