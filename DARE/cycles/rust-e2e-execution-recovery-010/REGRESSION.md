# REGRESSION

- Validate DAG and inspect ready task in a clean temporary project.
- Simulate Ralph timeout and expect exit 124 with persisted RUNNING state.
- Relaunch independent CLI process; verify RUNNING count.
- Explicitly reset task, complete under mock passing gates and verify DONE plus verification artifact.
- Confirm next action reports resolved.
- Check `dare info`, `dare support` and safe local support export.

All results pending GitHub CI execution.
