export function getTotalExecutionTime(tasks: any[]): number {
  console.log({tasks})
  if (tasks === undefined || tasks.length === 0) {
    return 0;
  }

  return tasks.reduce((acc, task) => {
    if (
      task.post_validate_output &&
      task.post_validate_output.execution_time_in_millis
    ) {
      return acc + task.post_validate_output.execution_time_in_millis;
    }

    return acc;
  }, 0);
}