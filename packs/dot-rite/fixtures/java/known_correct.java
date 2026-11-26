// Test-only known-correct Java fixture for dot-rite.
public class Solution {
    public static long dot_sparse(long[][] a, long[][] b) {
        java.util.Map<Long, Long> da = new java.util.HashMap<>();
        for (long[] r : a) da.put(r[0], r[1]);
        java.util.Map<Long, Long> db = new java.util.HashMap<>();
        for (long[] r : b) db.put(r[0], r[1]);
        long total = 0;
        for (java.util.Map.Entry<Long, Long> e : da.entrySet()) {
            Long other = db.get(e.getKey());
            if (other != null) total += e.getValue() * other;
        }
        return total;
    }
}
