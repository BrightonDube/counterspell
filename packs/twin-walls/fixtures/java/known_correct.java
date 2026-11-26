// Test-only known-correct Java fixture for twin-walls.
public class Solution {
    public static long twin_walls(long[][] edges, long n) {
        int size = (int) n;
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
        int[] colour = new int[size];
        java.util.Arrays.fill(colour, -1);
        for (int s = 0; s < size; s += 1) {
            if (colour[s] != -1) {
                continue;
            }
            colour[s] = 0;
            java.util.ArrayDeque<Integer> stack = new java.util.ArrayDeque<>();
            stack.push(s);
            while (!stack.isEmpty()) {
                int x = stack.pop();
                for (int y : adj.get(x)) {
                    if (colour[y] == -1) {
                        colour[y] = 1 - colour[x];
                        stack.push(y);
                    } else if (colour[y] == colour[x]) {
                        return 0;
                    }
                }
            }
        }
        return 1;
    }
}
