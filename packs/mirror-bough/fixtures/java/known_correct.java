// Test-only known-correct Java fixture for mirror-bough.
public class Solution {
    static long kid(long[] arr, int i, int n) {
        return i < arr.length && arr[i] >= 0 && arr[i] < n ? arr[i] : -1;
    }
    public static long mirror_check(long[] vals, long[] left, long[] right) {
        int n = vals.length;
        if (n == 0) return 1;
        java.util.Set<String> seen = new java.util.HashSet<>();
        java.util.ArrayDeque<long[]> work = new java.util.ArrayDeque<>();
        work.push(new long[]{kid(left, 0, n), kid(right, 0, n)});
        while (!work.isEmpty()) {
            long[] cur = work.pop();
            String key = cur[0] + "," + cur[1];
            if (seen.contains(key)) continue;
            seen.add(key);
            if (cur[0] == -1 && cur[1] == -1) continue;
            if (cur[0] == -1 || cur[1] == -1 || vals[(int) cur[0]] != vals[(int) cur[1]]) return 0;
            work.push(new long[]{kid(left, (int) cur[0], n), kid(right, (int) cur[1], n)});
            work.push(new long[]{kid(right, (int) cur[0], n), kid(left, (int) cur[1], n)});
        }
        return 1;
    }
}
