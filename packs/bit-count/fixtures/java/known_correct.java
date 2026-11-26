// Test-only known-correct Java fixture for bit-count.
public class Solution {
    public static long bit_count(long n) {
        long count = 0;
        while (n > 0) {
            count += n & 1;
            n >>= 1;
        }
        return count;
    }
}
