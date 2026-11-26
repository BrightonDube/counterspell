// Test-only known-correct Java fixture for canopy-height.
public class Solution {
    public static long tree_height(long[] vals, long[] left, long[] right) {
        int n = vals.length;
        if (n == 0) return 0;
        long best = 0;
        java.util.Set<Integer> seen = new java.util.HashSet<>();
        java.util.ArrayDeque<int[]> stack = new java.util.ArrayDeque<>();
        stack.push(new int[]{0, 1});
        while (!stack.isEmpty()) {
            int[] cur = stack.pop();
            if (seen.contains(cur[0])) continue;
            seen.add(cur[0]);
            if (cur[1] > best) best = cur[1];
            long lo = cur[0] < left.length && left[cur[0]] >= 0 && left[cur[0]] < n ? left[cur[0]] : -1;
            long hi = cur[0] < right.length && right[cur[0]] >= 0 && right[cur[0]] < n ? right[cur[0]] : -1;
            if (lo != -1) stack.push(new int[]{(int) lo, cur[1] + 1});
            if (hi != -1) stack.push(new int[]{(int) hi, cur[1] + 1});
        }
        return best;
    }
}
