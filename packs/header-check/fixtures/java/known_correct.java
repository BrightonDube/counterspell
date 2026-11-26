// Test-only known-correct Java fixture for header-check.
public class Solution {
    public static long header_check(long[] env) {
        long v = env[0], s = env[1], q = env[2], l = env[3];
        boolean ok = (v == 1 || v == 2) && s >= 0 && s <= 255 && q >= 0 && q <= 65535 && l >= 0 && l <= 1500;
        return ok ? 1 : 0;
    }
}
