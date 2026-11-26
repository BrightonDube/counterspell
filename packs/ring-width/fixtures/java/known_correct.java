// Test-only known-correct Java fixture for ring-width.
public class Solution {
    public static long widest_ring(long[] vals, long[] left, long[] right) {
        int n = vals.length;
        if (n == 0) return 0;
        java.util.Map<Integer, Long> counts = new java.util.HashMap<>();
        java.util.Set<Integer> seen = new java.util.HashSet<>();
        java.util.ArrayDeque<int[]> queue = new java.util.ArrayDeque<>();
        queue.offer(new int[]{0, 0});
        while (!queue.isEmpty()) {
            int[] cur = queue.poll();
            if (seen.contains(cur[0])) continue;
            seen.add(cur[0]);
            counts.merge(cur[1], 1L, Long::sum);
            long lo = cur[0] < left.length && left[cur[0]] >= 0 && left[cur[0]] < n ? left[cur[0]] : -1;
            long hi = cur[0] < right.length && right[cur[0]] >= 0 && right[cur[0]] < n ? right[cur[0]] : -1;
            if (lo != -1) queue.offer(new int[]{(int) lo, cur[1] + 1});
            if (hi != -1) queue.offer(new int[]{(int) hi, cur[1] + 1});
        }
        return counts.values().stream().mapToLong(Long::longValue).max().orElse(0);
    }
}
