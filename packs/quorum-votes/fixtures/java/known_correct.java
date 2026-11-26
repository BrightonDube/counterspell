// Test-only known-correct Java fixture for quorum-votes.
public class Solution {
    public static long quorum_reached(long n, long votes) {
        return votes >= n / 2 + 1 ? 1 : 0;
    }
}
