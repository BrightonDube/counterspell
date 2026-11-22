// Test-only deep-recursion Java fixture (language-drivers T14).
// Linear recursion to depth n: the shape linked-list/tree-depth problems take.
// Used by the -Xss sizing evidence in java-stack-sizing.md; the smallest
// candidate that passes this fixture repeatedly is the configured stack.
public class Solution {
    public static long depth(long n) {
        return n <= 0 ? 0 : 1 + depth(n - 1);
    }
}
