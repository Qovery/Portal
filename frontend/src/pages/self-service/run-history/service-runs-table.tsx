import Table from "@/components/Table";
import { classNames, millisToHumanTime } from "@/lib/utils.ts";
import { ServiceRun } from "@/types/catalog.type";
import { createColumnHelper } from "@tanstack/react-table";
import dayjs from "dayjs";
import { useMemo } from "react";
import { Link } from "react-router-dom";
import relativeTime from "dayjs/plugin/relativeTime";
import clsx from "clsx";
import { RunStatus } from "@/enums/run-status.enum";

dayjs.extend(relativeTime);

function getStatusStyle(status: RunStatus): string {
  switch (status) {
    case RunStatus.QUEUED:
      return "text-orange-400 bg-orange-400/10";
    case RunStatus.RUNNING:
      return "text-cyan-400 bg-cyan-400/10";
    case RunStatus.SUCCESS:
      return "text-green-400 bg-green-400/10";
    case RunStatus.FAILURE:
      return "text-rose-400 bg-rose-400/10";
    default:
      return "text-gray-400 bg-gray-400/10";
  }
}

function getTotalExecutionTime(tasks: any[]): number {
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

function totalSuccessTasks(tasks: any[]): number {
  if (tasks === undefined || tasks.length === 0) {
    return 0;
  }

  return tasks.reduce((acc, task) => {
    if (task.status === RunStatus.SUCCESS) {
      return acc + 1;
    }

    return acc;
  }, 0);
}

interface ServiceRunsTable {
  rows: ServiceRun[];
  isLoading: boolean;
  error: any;
}

interface ServiceRunRow {
  createdAt: string;
  executionTime: string;
  id: string;
  serviceSlug: string;
  status: string;
  tasks: number;
}

const columnHelper = createColumnHelper<ServiceRunRow>();

export default function ServiceRunsTable({
  rows,
  isLoading,
  error,
}: ServiceRunsTable) {
  const mappedRows = useMemo(
    () =>
      rows.map((row) => ({
        createdAt: dayjs(row.created_at).fromNow(),
        executionTime: millisToHumanTime(getTotalExecutionTime(row.tasks)),
        id: row.id,
        serviceSlug: row.input_payload.name,
        status: row.status,
        tasks: totalSuccessTasks(row.tasks) / row.tasks.length,
      })),
    [rows],
  );

  const columns = useMemo(
    () => [
      columnHelper.accessor("createdAt", {
        id: "date",
        header: () => "Date",
        cell: (cellProps) => cellProps.row.original.createdAt,
      }),
      columnHelper.accessor("serviceSlug", {
        id: "service",
        header: () => "Service",
        cell: (cellProps) => cellProps.row.original.serviceSlug,
      }),
      columnHelper.display({
        id: "status",
        header: () => "Status",
        cell: (cellProps) => {
          return (
            <div className="flex items-center justify-end gap-x-2 sm:justify-start">
              <time
                className="text-gray-400 sm:hidden"
                dateTime={cellProps.row.original.createdAt}
              >
                {cellProps.row.original.createdAt}
              </time>
              <div
                className={classNames(
                  getStatusStyle(cellProps.row.original.status),
                  "flex-none rounded-full p-1",
                )}
              >
                <div className="size-1.5 rounded-full bg-current" />
              </div>
              <div className="hidden text-sm text-gray-500 sm:block">
                {cellProps.row.original.status}
              </div>
            </div>
          );
        },
      }),
      columnHelper.accessor("tasks", {
        id: "total",
        header: () => "Tasks",
        cell: (cellProps) => cellProps.row.original.tasks,
      }),
      columnHelper.accessor("executionTime", {
        id: "lastUpdated",
        header: () => "Execution Time",
        cell: (cellProps) => cellProps.row.original.executionTime,
      }),
      columnHelper.display({
        id: "edit",
        header: () => "Actions",
        cell: (cellProps) => {
          return (
            <Link to={`/self-service/${cellProps.row.original.id}/details`} className="text-indigo-600 hover:underline">
              Show details{" "}
              <span className="sr-only">
                , {cellProps.row.original.serviceSlug}
              </span>
            </Link>
          );
        },
      }),
    ],
    [],
  );

  return (
    <div className={clsx(isLoading && "blur-sm")}>
      <Table columns={columns} data={mappedRows} />
    </div>
  );
}
