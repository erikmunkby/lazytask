<!---
This prompt is returned as next_step when accumulated learnings exceed the configured threshold.
Goal: nudge the agent to suggest a dedicated learning review session to the user.
-->
!!IMPORTANT: Time to learn!
There are enough accumulated learnings to justify a review pass.
Suggest to the user that they start a fresh conversation (clear context first) dedicated to a lazytask learning review.
The learning review command is `lt learn --review` — it works best with a clean context so the agent can focus entirely on turning learnings into concrete improvements.
