// Test-only known-correct Java fixture for digit-rite.
public class Solution {
    public static long digit_rite(long n) {
        long orig = n;
        long count = 0;
        while (n > 0) {
            long d = n % 10;
            if (d != 0 && orig % d == 0) {
                count += 1;
            }
            n /= 10;
        }
        return count;
    }
}
