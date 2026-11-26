// Test-only known-correct Java fixture for chain-length.
public class Solution {
    public static long chain_length(long[] nxt) {
        long i = nxt.length > 0 ? 0 : -1, count = 0;
        while (i != -1) { count += 1; i = nxt[(int) i]; }
        return count;
    }
}
