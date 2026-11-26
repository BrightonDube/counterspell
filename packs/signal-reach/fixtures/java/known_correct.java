// Test-only known-correct Java fixture for signal-reach.
public class Solution {
    public static long reach_count(long[][] edges, long n) {
        int size = (int) n;
        if (size <= 0) {
            return 0;
        }
        java.util.List<java.util.List<Integer>> adj = new java.util.ArrayList<>();
        for (int i = 0; i < size; i += 1) {
            adj.add(new java.util.ArrayList<>());
        }
        for (long[] e : edges) {
            int a = (int) e[0];
            int b = (int) e[1];
            if (a >= 0 && a < size && b >= 0 && b < size) {
                adj.get(a).add(b);
                adj.get(b).add(a);
            }
        }
        boolean[] seen = new boolean[size];
        seen[0] = true;
        java.util.ArrayDeque<Integer> stack = new java.util.ArrayDeque<>();
        stack.push(0);
        long count = 0;
        while (!stack.isEmpty()) {
            int x = stack.pop();
            count += 1;
            for (int y : adj.get(x)) {
                if (!seen[y]) {
                    seen[y] = true;
                    stack.push(y);
                }
            }
        }
        return count;
    }
}
