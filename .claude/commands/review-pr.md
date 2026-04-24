Review the pull request: $ARGUMENTS

Follow these steps carefully. Use the `gh` CLI for all GitHub interactions.

## Step 1: Resolve the PR

Parse `$ARGUMENTS` to determine the PR. It can be:

- A full URL like `https://github.com/owner/repo/pull/123`
- A `owner/repo#123` reference
- A bare number like `123` (use the current repo)
- A description — search for it with `gh pr list --search "<description>" --limit 5` and pick the best match

Once resolved, fetch the PR metadata:

```bash
gh pr view <PR> --json number,title,body,author,state,baseRefName,headRefName,url,labels,milestone,additions,deletions,changedFiles,createdAt,updatedAt,mergedAt,reviewDecision,reviews,assignees
```

## Step 2: Gather the diff

Get the full diff of the PR:

```bash
gh pr diff <PR>
```

If the diff is very large (>3000 lines), focus on the most important files first and summarize the rest.

## Step 3: Collect PR discussion context

Fetch all comments and review threads:

```bash
gh api repos/{owner}/{repo}/pulls/{number}/comments --paginate
gh api repos/{owner}/{repo}/issues/{number}/comments --paginate
gh api repos/{owner}/{repo}/pulls/{number}/reviews --paginate
```

Pay attention to:

- Reviewer feedback and requested changes
- Author responses and explanations
- Any unresolved conversations
- Approval or rejection status

## Step 4: Find and read linked issues

Look for issue references in:

- The PR body (patterns like `#123`, `fixes #123`, `closes #123`, `resolves #123`)
- The PR branch name (patterns like `issue-123`, `fix/123`)
- Commit messages

For each linked issue, fetch its content:

```bash
gh issue view <number> --json title,body,comments,labels,state
```

Read through issue comments to understand the original problem, user reports, and any discussed solutions.

## Step 5: Analyze and Validate (The "Superior" Audit)
In addition to standard checks, analyze the PR against LiteLLM-rs specific gates:

1. **Schema Integrity**: If `specs/*.json` were changed, did the author run `cargo run --bin audit-specs`? Are the generated types in the diff consistent with the spec changes?
2. **Zero-Copy Adherence**: Does the code use `Cow<'a, str>` or `&str` for high-frequency paths (like model names)? Flag unnecessary `String` allocations.
3. **Bifrost Compliance**: Does the PR maintain `x-bf-vk` header pass-through? Does it break the Provisioner logic?
4. **Testing Rigor**: Does it include a `wiremock` integration test if a new provider was added?
5. **No Emoticons**: Audit all comments and documentation for prohibited emoticons.
6. **Performance Check**: If `src/models.rs` was touched, is there a `cargo bench` summary in the PR comments?

## Step 6: Produce the Review Summary
(Use the structured format provided, but add the following section)

#### Superior Standards Audit
- **Zero-Copy:** [PASS/FAIL/NOT APPLICABLE]
- **Spec Integrity:** [PASS/FAIL]
- **Bifrost Native:** [PASS/FAIL]
- **Emoticon-Free:** [PASS/FAIL]

#### Verdict
(APPROVE / REQUEST CHANGES / NEEDS DISCUSSION)
