// Test-only known-correct Java fixture for wild-lexicon.
public class Solution {
    private static boolean matchWord(String w, String p) {
        boolean[][] dp = new boolean[w.length() + 1][p.length() + 1];
        dp[0][0] = true;
        for (int j = 1; j <= p.length(); j += 1) {
            if (p.charAt(j - 1) == '*') {
                dp[0][j] = dp[0][j - 1];
            }
        }
        for (int i = 1; i <= w.length(); i += 1) {
            for (int j = 1; j <= p.length(); j += 1) {
                dp[i][j] =
                    (p.charAt(j - 1) == '*' && (dp[i][j - 1] || dp[i - 1][j]))
                    || (p.charAt(j - 1) == w.charAt(i - 1) && dp[i - 1][j - 1]);
            }
        }
        return dp[w.length()][p.length()];
    }

    public static long count_matches(String words, String pattern) {
        java.util.Set<String> ws = new java.util.HashSet<>();
        for (String w : words.split(",", -1)) {
            if (!w.isEmpty()) {
                ws.add(w);
            }
        }
        long ans = 0;
        for (String w : ws) {
            if (matchWord(w, pattern)) {
                ans += 1;
            }
        }
        return ans;
    }
}
