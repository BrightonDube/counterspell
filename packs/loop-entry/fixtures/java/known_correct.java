// Test-only known-correct Java fixture for loop-entry.
public class Solution {
    public static long loop_entry(long[] nxt) {
        java.util.Set<Long> seen = new java.util.HashSet<>();
        long i = nxt.length > 0 ? 0 : -1;
        while (i != -1 && !seen.contains(i)) { seen.add(i); i = nxt[(int) i]; }
        return i;
    }
}
