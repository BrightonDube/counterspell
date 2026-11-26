// Test-only known-correct Java fixture for alloc-rite.
public class Solution {
    public static long alloc_rite(long[] blocks, long[] reqs) {
        long[] slots = blocks.clone();
        long granted = 0;
        for (long r : reqs) {
            for (int i = 0; i < slots.length; i++) {
                if (slots[i] >= r) { slots[i] -= r; granted += 1; break; }
            }
        }
        return granted;
    }
}
