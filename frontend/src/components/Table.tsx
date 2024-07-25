import { ArrowLongUpIcon } from "@heroicons/react/24/outline"
import {
  ColumnDef,
  OnChangeFn,
  Row,
  SortingState,
  Updater,
  flexRender,
  getCoreRowModel,
  getExpandedRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table"
import clsx from "clsx"
import { Fragment, useEffect } from "react"

// https://github.com/TanStack/table/issues/4382
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type TableColumns<T> = ColumnDef<T, any>[]

export type TableRowMode = "default" | "condensed"

export interface TableProps<T> {
  data: T[]
  columns: TableColumns<T>
  renderSubComponent?: (props: { row: Row<T> }) => React.ReactElement
  sorting?: SortingState
  onSortingChange?: OnChangeFn<SortingState>
  manualSorting?: boolean

  /**
   * If using expandable rows, this function will be called for each row to determine if it should be expanded or not.
   */
  expandCondition?: (row: Row<T>) => boolean
}

/** Only used for client side sorting */
export function sortHelper<T>(rows: T[], sorting: SortingState) {
  const inputRows = [...rows]

  if (sorting.length > 0) {
    inputRows.sort((a, b) => {
      const sort = sorting[0]
      const aVal = a[sort.id as keyof T]
      const bVal = b[sort.id as keyof T]

      if (aVal === bVal) {
        return 0
      }

      if (sort.desc) {
        return aVal! > bVal! ? -1 : 1
      } else {
        return aVal! > bVal! ? 1 : -1
      }
    })
  }

  return inputRows
}

export function Table<T>(props: TableProps<T>) {
  const { data, columns, renderSubComponent, expandCondition } = props

  const sortingChanged: OnChangeFn<SortingState> = (
    sortingState: Updater<SortingState>,
  ) => {
    props.onSortingChange && props.onSortingChange(sortingState)
  }

  const table = useReactTable<T>({
    data,
    columns,
    state: {
      sorting: props.sorting,
    },
    getCoreRowModel: getCoreRowModel(),
    getExpandedRowModel: getExpandedRowModel(),
    getSortedRowModel: getSortedRowModel(),
    manualSorting: props.manualSorting ?? false,
    onSortingChange: sortingChanged,
  })

  useEffect(() => {
    if (expandCondition) {
      const rows = table.getRowModel().rows
      for (const row of rows) {
        const val = expandCondition(row)
        row.toggleExpanded(val)
      }
    }
  }, [table, expandCondition])

  return (
    <table className="min-w-full divide-y divide-gray-300">
      <thead>
        {table.getHeaderGroups().map((headerGroup) => (
          <tr key={headerGroup.id}>
            {headerGroup.headers.map((header) => (
              <th
                key={header.id}
                scope="col"
                className={clsx(
                  "py-3.5 pl-4 pr-3 text-left text-sm font-semibold text-gray-900 first:rounded-tl-md last:rounded-tr-md",
                  header.column.getCanSort() && "cursor-pointer select-none",
                )}
                onClick={header.column.getToggleSortingHandler()}
                style={{
                  width: `${header.getSize()}px`,
                }}
              >
                <div className="group inline-flex items-center">
                  {header.isPlaceholder
                    ? null
                    : flexRender(
                        header.column.columnDef.header,
                        header.getContext(),
                      )}
                  {{
                    asc: (
                      <span className="ml-2 flex-none rounded text-gray-500 group-hover:text-gray-900">
                        <ArrowLongUpIcon
                          className="size-4"
                          aria-hidden="true"
                        />
                      </span>
                    ),
                    desc: (
                      <span className="ml-2 flex-none rounded text-gray-500 group-hover:text-gray-900">
                        <ArrowLongUpIcon
                          className="size-4 rotate-180"
                          aria-hidden="true"
                        />
                      </span>
                    ),
                  }[header.column.getIsSorted() as string] ?? null}
                </div>
              </th>
            ))}
          </tr>
        ))}
      </thead>
      <tbody className="divide-y divide-gray-200">
        {table.getRowModel().rows.map((row) => {
          return (
            <Fragment key={row.id}>
              <tr>
                {/* first row is a normal row */}
                {row.getVisibleCells().map((cell) => {
                  return (
                    <td
                      key={cell.id}
                      className={clsx(
                        "whitespace-nowrap py-5 pl-4 pr-3 align-middle text-sm",
                      )}
                      style={{
                        width: `${cell.column.getSize()}px`,
                        textAlign:
                          cell.column.columnDef.id === "actions"
                            ? "right"
                            : "left",
                      }}
                    >
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext(),
                      )}
                    </td>
                  )
                })}
              </tr>
              {row.getIsExpanded() && renderSubComponent && (
                <tr className={clsx("border-none")}>
                  {/* 2nd row is a custom 1 cell row */}
                  <td colSpan={row.getVisibleCells().length}>
                    {renderSubComponent({ row })}
                  </td>
                </tr>
              )}
            </Fragment>
          )
        })}
      </tbody>
    </table>
  )
}

export default Table
