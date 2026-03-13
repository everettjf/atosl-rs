int helper_add(int lhs, int rhs) {
    return lhs + rhs;
}

__attribute__((noinline))
int golden_alpha(void) {
    return helper_add(20, 22);
}

__attribute__((noinline))
int golden_beta(void) {
    return golden_alpha() + 1;
}

int main(void) {
    return golden_beta();
}
