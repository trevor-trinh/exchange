"use client";

import {
  ColumnDef,
  flexRender,
  getCoreRowModel,
  getSortedRowModel,
  SortingState,
  useReactTable,
} from "@tanstack/react-table";
import { useState, useRef, useEffect } from "react";
import { TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";

interface DataTableProps<TData, TValue> {
  columns: ColumnDef<TData, TValue>[];
  data: TData[];
  emptyMessage?: string;
  headerAction?: React.ReactNode;
}

export function DataTable<TData, TValue>({
  columns,
  data,
  emptyMessage = "No data available.",
  headerAction,
}: DataTableProps<TData, TValue>) {
  const getDefaultSortColumn = () => {
    const firstColumn = columns[0];
    if (!firstColumn) return "";

    // Check if column has accessorKey property (ColumnDef with accessor)
    if ("accessorKey" in firstColumn && typeof firstColumn.accessorKey === "string") {
      return firstColumn.accessorKey;
    }

    // Otherwise use id if available
    return firstColumn.id || "";
  };

  const [sorting, setSorting] = useState<SortingState>([{ id: getDefaultSortColumn(), desc: true }]);
  const scrollContainerRef = useRef<HTMLDivElement>(null);

  // eslint-disable-next-line react-hooks/incompatible-library
  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    onSortingChange: setSorting,
    enableSortingRemoval: false,
    enableMultiSort: false,
    state: {
      sorting,
    },
  });

  // Prevent scroll propagation to parent containers
  useEffect(() => {
    const scrollContainer = scrollContainerRef.current;
    if (!scrollContainer) return;

    const handleWheel = (e: WheelEvent) => {
      const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
      const isScrollingUp = e.deltaY < 0;
      const isScrollingDown = e.deltaY > 0;

      // At the top and scrolling up - allow propagation
      if (isScrollingUp && scrollTop === 0) {
        return;
      }

      // At the bottom and scrolling down - allow propagation
      if (isScrollingDown && scrollTop + clientHeight >= scrollHeight) {
        return;
      }

      // Otherwise, stop propagation to keep scroll contained
      e.stopPropagation();
    };

    scrollContainer.addEventListener("wheel", handleWheel, { passive: false });

    return () => {
      scrollContainer.removeEventListener("wheel", handleWheel);
    };
  }, []);

  return (
    <div ref={scrollContainerRef} className="h-full overflow-auto relative">
      <table className="w-full caption-bottom text-sm">
        <TableHeader
          className="sticky top-0 z-10 border-b border-border/50 bg-neutral-800"
          style={{
            backgroundImage: `
                    url("data:image/svg+xml,%3Csvg width='4' height='4' viewBox='0 0 4 4' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath d='M0 0h1v1H0zM2 2h1v1H2z' fill='%23000000' fill-opacity='0.1'/%3E%3Cpath d='M1 0h1v1H1zM3 2h1v1H3zM0 2h1v1H0zM2 0h1v1H2zM1 2h1v1H1zM3 0h1v1H3z' fill='%23ffffff' fill-opacity='0.05'/%3E%3C/svg%3E"),
                    repeating-linear-gradient(0deg, rgba(0, 0, 0, 0.03) 0px, rgba(0, 0, 0, 0.03) 1px, transparent 1px, transparent 2px)
                  `,
            backgroundBlendMode: "overlay, normal",
          }}
        >
          {table.getHeaderGroups().map((headerGroup) => (
            <TableRow key={headerGroup.id} className="border-none hover:bg-transparent relative">
              {headerGroup.headers.map((header) => {
                return (
                  <TableHead
                    key={header.id}
                    className="font-semibold text-foreground px-3"
                    style={{ width: header.getSize() !== 150 ? header.getSize() : undefined }}
                  >
                    {header.isPlaceholder ? null : (
                      <div
                        className={
                          header.column.getCanSort()
                            ? "cursor-pointer select-none flex items-center justify-between gap-1 hover:text-primary transition-colors w-full group"
                            : "w-full"
                        }
                        onClick={header.column.getToggleSortingHandler()}
                      >
                        <div className="flex-1">{flexRender(header.column.columnDef.header, header.getContext())}</div>
                        {header.column.getCanSort() && header.column.getIsSorted() && (
                          <span className="relative shrink-0 w-3 h-3 flex items-center justify-center">
                            <span
                              className={`absolute text-xs transition-all duration-300 ease-out ${
                                header.column.getIsSorted() === "asc"
                                  ? "rotate-0 text-primary"
                                  : "rotate-180 text-primary"
                              }`}
                            >
                              ↑
                            </span>
                          </span>
                        )}
                      </div>
                    )}
                  </TableHead>
                );
              })}
              {headerAction && <div className="absolute right-3 top-0 h-full flex items-center">{headerAction}</div>}
            </TableRow>
          ))}
        </TableHeader>
        <TableBody>
          {table.getRowModel().rows?.length ? (
            table.getRowModel().rows.map((row) => (
              <TableRow
                key={row.id}
                data-state={row.getIsSelected() && "selected"}
                className="border-border/30 hover:bg-primary/5 transition-colors"
              >
                {row.getVisibleCells().map((cell) => (
                  <TableCell key={cell.id} className="px-3">
                    {flexRender(cell.column.columnDef.cell, cell.getContext())}
                  </TableCell>
                ))}
              </TableRow>
            ))
          ) : (
            <TableRow>
              <TableCell colSpan={columns.length} className="h-24 text-center">
                <p className="text-muted-foreground text-sm">{emptyMessage}</p>
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </table>
    </div>
  );
}
