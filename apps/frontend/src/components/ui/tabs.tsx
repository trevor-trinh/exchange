"use client";

import * as React from "react";
import * as TabsPrimitive from "@radix-ui/react-tabs";

import { cn } from "@/lib/utils";

function Tabs({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.Root>) {
  return <TabsPrimitive.Root data-slot="tabs" className={cn("flex flex-col gap-2", className)} {...props} />;
}

function TabsList({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.List>) {
  return (
    <TabsPrimitive.List
      data-slot="tabs-list"
      className={cn(
        "bg-muted text-muted-foreground inline-flex h-9 w-fit items-center justify-center p-[3px] dither",
        className
      )}
      {...props}
    />
  );
}

function TabsTrigger({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.Trigger>) {
  return (
    <TabsPrimitive.Trigger
      data-slot="tabs-trigger"
      style={{ borderLeft: "none", borderRight: "none", borderTop: "none" }}
      className={cn(
        "text-foreground dark:text-muted-foreground inline-flex h-full flex-1 items-center justify-center gap-1.5 border-b-[3px] border-b-transparent px-2 py-2 text-sm font-medium whitespace-nowrap transition-all duration-300 focus-visible:ring-[3px] focus-visible:outline-1 disabled:pointer-events-none disabled:opacity-50 [&_svg]:pointer-events-none [&_svg]:shrink-0 [&_svg:not([class*='size-'])]:size-4 relative cursor-pointer",
        "hover:text-primary/90 hover:bg-primary/5 hover:shadow-[0_1px_0px_0px_rgba(180,150,255,0.3)]",
        "active:scale-[0.98] active:bg-primary/10",
        "data-[state=active]:text-primary data-[state=active]:border-b-primary data-[state=active]:shadow-[0_3px_0px_0px_rgba(180,150,255,0.6),0_1px_0px_0px_rgba(255,255,255,0.5)] data-[state=active]:bg-primary/5",
        "focus-visible:border-ring focus-visible:ring-ring/50 focus-visible:outline-ring",
        className
      )}
      {...props}
    />
  );
}

function TabsContent({ className, ...props }: React.ComponentProps<typeof TabsPrimitive.Content>) {
  return <TabsPrimitive.Content data-slot="tabs-content" className={cn("flex-1 outline-none", className)} {...props} />;
}

export { Tabs, TabsList, TabsTrigger, TabsContent };
