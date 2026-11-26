// Test-only known-correct Java fixture for ember-diameter.
public class Solution {
    public static long tree_diameter(long[] vals, long[] left, long[] right) {
        int n = vals.length;
        if (n == 0) return 0;
        long[] heights = new long[n];
        java.util.Arrays.fill(heights, -1);
        java.util.Set<Integer> seen = new java.util.HashSet<>();
        java.util.ArrayDeque<int[]> stack = new java.util.ArrayDeque<>();
        stack.push(new int[]{0, 0});
        long best = 0;
        while (!stack.isEmpty()) {
            int[] cur = stack.pop();
            int i = cur[0];
            if (seen.contains(i)) continue;
            if (cur[1] == 0) {
                stack.push(new int[]{i, 1});
                for (long[] arr : new long[][]{left, right}) {
                    int k = i < arr.length && arr[i] >= 0 && arr[i] < n ? (int) arr[i] : -1;
                    if (k != -1) stack.push(new int[]{k, 0});
                }
                continue;
            }
            seen.add(i);
            int lo = i < left.length && left[i] >= 0 && left[i] < n ? (int) left[i] : -1;
            int hi = i < right.length && right[i] >= 0 && right[i] < n ? (int) right[i] : -1;
            long lh = lo != -1 && heights[lo] >= 0 ? heights[lo] + 1 : 0;
            long rh = hi != -1 && heights[hi] >= 0 ? heights[hi] + 1 : 0;
            if (lh + rh > best) best = lh + rh;
            heights[i] = Math.max(lh, rh);
        }
        return best;
    }
}
