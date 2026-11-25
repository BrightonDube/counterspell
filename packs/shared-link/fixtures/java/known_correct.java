// Test-only known-correct Java fixture for shared-link.
public class Solution {
    public static long shared_link(long[] nxt, long ha, long hb) {
        java.util.Set<Long> seen = new java.util.HashSet<>();
        for (long i = ha; i != -1; i = nxt[(int) i]) { seen.add(i); }
        long j = hb;
        while (j != -1 && !seen.contains(j)) { j = nxt[(int) j]; }
        return j;
    }
}
