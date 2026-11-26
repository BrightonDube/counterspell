// Test-only known-correct Java fixture for sigil-bridge.
public class Solution {
    static int stamp;
    static int bridges;
    static void dfs(int u, int entryArc, int[][] adjArcs, int[][] adjNodes, int[] disc, int[] low) {
        disc[u] = stamp;
        low[u] = stamp;
        stamp += 1;
        for (int i = 0; i < adjNodes[u].length; i += 1) {
            int v = adjNodes[u][i];
            int arc = adjArcs[u][i];
            if (arc == entryArc) {
                continue;
            }
            if (disc[v] == -1) {
                dfs(v, arc, adjArcs, adjNodes, disc, low);
                low[u] = Math.min(low[u], low[v]);
                if (low[v] > disc[u]) {
                    bridges += 1;
                }
            } else {
                low[u] = Math.min(low[u], disc[v]);
            }
        }
    }
    public static long bridge_count(long[][] edges, long n) {
        int size = (int) n;
        java.util.List<java.util.List<int[]>> pairs = new java.util.ArrayList<>();
        for (int i = 0; i < size; i += 1) {
            pairs.add(new java.util.ArrayList<>());
        }
        int arc = 0;
        for (long[] e : edges) {
            int a = (int) e[0];
            int b = (int) e[1];
            if (a >= 0 && a < size && b >= 0 && b < size && a != b) {
                pairs.get(a).add(new int[]{b, arc});
                pairs.get(b).add(new int[]{a, arc});
                arc += 1;
            }
        }
        int[][] adjNodes = new int[size][];
        int[][] adjArcs = new int[size][];
        for (int i = 0; i < size; i += 1) {
            java.util.List<int[]> list = pairs.get(i);
            int[] nodes = new int[list.size()];
            int[] arcs = new int[list.size()];
            for (int j = 0; j < list.size(); j += 1) {
                nodes[j] = list.get(j)[0];
                arcs[j] = list.get(j)[1];
            }
            adjNodes[i] = nodes;
            adjArcs[i] = arcs;
        }
        int[] disc = new int[size];
        java.util.Arrays.fill(disc, -1);
        int[] low = new int[size];
        stamp = 0;
        bridges = 0;
        for (int s = 0; s < size; s += 1) {
            if (disc[s] == -1) {
                dfs(s, -1, adjArcs, adjNodes, disc, low);
            }
        }
        return bridges;
    }
}
