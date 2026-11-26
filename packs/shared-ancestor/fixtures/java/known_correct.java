// Test-only known-correct Java fixture for shared-ancestor.
public class Solution {
    public static long first_ancestor(long[] vals, long[] left, long[] right, long a, long b) {
        int n = vals.length;
        if (n == 0 || a < 0 || a >= n || b < 0 || b >= n) return -1;
        java.util.Map<Integer, Integer> parent = new java.util.HashMap<>();
        java.util.Set<Integer> seen = new java.util.HashSet<>();
        java.util.ArrayDeque<Integer> stack = new java.util.ArrayDeque<>();
        stack.push(0);
        while (!stack.isEmpty()) {
            int i = stack.pop();
            if (seen.contains(i)) continue;
            seen.add(i);
            for (long[] arr : new long[][]{left, right}) {
                int k = i < arr.length && arr[i] >= 0 && arr[i] < n ? (int) arr[i] : -1;
                if (k != -1 && !parent.containsKey(k)) { parent.put(k, i); stack.push(k); }
            }
        }
        if (!seen.contains((int) a) || !seen.contains((int) b)) return -1;
        java.util.Set<Integer> anc = new java.util.HashSet<>();
        for (int x = (int) a; x != -1; x = parent.getOrDefault(x, -1)) anc.add(x);
        for (int x = (int) b; x != -1; x = parent.getOrDefault(x, -1)) {
            if (anc.contains(x)) return x;
        }
        return -1;
    }
}
