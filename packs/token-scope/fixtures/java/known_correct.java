// Test-only known-correct Java fixture for token-scope.
public class Solution {
    public static long scope_ok(long[] required, long[] granted, long expiry, long now) {
        boolean alive = now < expiry;
        boolean covered = true;
        java.util.Set<Long> g = new java.util.HashSet<>();
        for (long s : granted) g.add(s);
        for (long s : required) if (!g.contains(s)) covered = false;
        return alive && covered ? 1 : 0;
    }
}
