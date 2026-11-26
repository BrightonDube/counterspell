// Test-only known-correct Java fixture for wrap-rite.
public class Solution {
    public static long wrap_rite(long[] marks, long b) {
        java.util.LinkedList<Long> ring = new java.util.LinkedList<>();
        long evictions = 0;
        for (long m : marks) {
            if (!ring.contains(m)) {
                if (ring.size() == b) { ring.pollFirst(); evictions += 1; }
                ring.addLast(m);
            }
        }
        return evictions;
    }
}
