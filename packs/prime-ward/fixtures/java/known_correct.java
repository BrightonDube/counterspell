// Test-only known-correct Java fixture for prime-ward.
public class Solution {
    public static long prime_ward(long n) {
        if (n < 2) {
            return 0;
        }
        boolean[] sieve = new boolean[(int) n + 1];
        java.util.Arrays.fill(sieve, true);
        sieve[0] = sieve[1] = false;
        for (long p = 2; p * p <= n; p += 1) {
            if (sieve[(int) p]) {
                for (long k = p * p; k <= n; k += p) {
                    sieve[(int) k] = false;
                }
            }
        }
        long count = 0;
        for (boolean s : sieve) {
            if (s) {
                count += 1;
            }
        }
        return count;
    }
}
