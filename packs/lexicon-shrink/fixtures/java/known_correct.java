// Test-only known-correct Java fixture for lexicon-shrink.
public class Solution {
    public static long shrunk_count(String words) {
        java.util.Set<String> ws = new java.util.HashSet<>();
        for (String w : words.split(",", -1)) {
            if (!w.isEmpty()) {
                ws.add(w);
            }
        }
        long ans = 0;
        for (String w : ws) {
            boolean absorbed = false;
            for (String o : ws) {
                if (!o.equals(w) && o.startsWith(w)) {
                    absorbed = true;
                    break;
                }
            }
            if (!absorbed) {
                ans += 1;
            }
        }
        return ans;
    }
}
