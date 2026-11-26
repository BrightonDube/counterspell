// Test-only known-correct Java fixture for meld-cost.
public class Solution {
    public static long meld_cost(long[] sizes) {
        java.util.PriorityQueue<Long> heap = new java.util.PriorityQueue<>();
        for (long s : sizes) {
            heap.add(s);
        }
        long cost = 0;
        while (heap.size() > 1) {
            long fused = heap.poll() + heap.poll();
            cost += fused;
            heap.add(fused);
        }
        return cost;
    }
}
