// Test-only known-correct Java fixture for cluster-step.
public class Solution {
    public static long cluster_step(long[] points, long[] centroids, long[] labels) {
        if (centroids.length == 0) return points.length;
        long moved = 0;
        for (int i = 0; i < points.length; i++) {
            int best = 0;
            for (int j = 1; j < centroids.length; j++) {
                if (Math.abs(points[i] - centroids[j]) < Math.abs(points[i] - centroids[best])) best = j;
            }
            if (i >= labels.length || labels[i] != best) moved++;
        }
        return moved;
    }
}
