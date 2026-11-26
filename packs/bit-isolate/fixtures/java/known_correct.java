// Test-only known-correct Java fixture for bit-isolate.
public class Solution {
    public static long bit_isolate(long n) {
        return n & (-n);
    }
}
