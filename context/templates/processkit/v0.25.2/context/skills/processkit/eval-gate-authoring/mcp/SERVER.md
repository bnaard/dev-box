# processkit-eval-gate-authoring MCP server

Tools:

- `collect_run_outputs(run_root?, limit?)`
- `codify_eval(category_id, name, eval_kind, description, judge?)`
- `calibrate_judge(judge_id, eval_artifact_id?, agreement?, sample_n?, evidence?)`
- `bind_eval_to_runs(eval_artifact_id, target, description?)`

The write tools create processkit Artifacts, Gates, Bindings, and
LogEntries through the normal entity/index/log path.

