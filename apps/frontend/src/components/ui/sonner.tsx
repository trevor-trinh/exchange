"use client";

import { CircleCheckIcon, InfoIcon, Loader2Icon, OctagonXIcon, TriangleAlertIcon } from "lucide-react";
import { useTheme } from "next-themes";
import { Toaster as Sonner, type ToasterProps } from "sonner";

const Toaster = ({ ...props }: ToasterProps) => {
  const { theme = "system" } = useTheme();

  return (
    <Sonner
      theme={theme as ToasterProps["theme"]}
      className="toaster group"
      icons={{
        success: <CircleCheckIcon className="size-4" />,
        info: <InfoIcon className="size-4" />,
        warning: <TriangleAlertIcon className="size-4" />,
        error: <OctagonXIcon className="size-4" />,
        loading: <Loader2Icon className="size-4 animate-spin" />,
      }}
      toastOptions={{
        classNames: {
          toast: "bg-card/95 backdrop-blur-xl border-border/50 shadow-lg",
          title: "text-foreground",
          description: "text-muted-foreground",
          success: "bg-card/95 border-primary/30 text-foreground",
          error: "bg-card/95 border-destructive/50 text-foreground",
          warning: "bg-card/95 border-yellow-500/50 text-foreground",
          info: "bg-card/95 border-primary/30 text-foreground",
        },
      }}
      style={
        {
          "--normal-bg": "hsl(var(--card))",
          "--normal-text": "hsl(var(--foreground))",
          "--normal-border": "hsl(var(--border))",
          "--success-bg": "hsl(var(--card))",
          "--success-text": "hsl(var(--foreground))",
          "--success-border": "hsl(var(--primary) / 0.3)",
          "--error-bg": "hsl(var(--card))",
          "--error-text": "hsl(var(--foreground))",
          "--error-border": "hsl(var(--destructive) / 0.5)",
          "--warning-bg": "hsl(var(--card))",
          "--warning-text": "hsl(var(--foreground))",
          "--warning-border": "hsl(45 93% 47% / 0.5)",
          "--info-bg": "hsl(var(--card))",
          "--info-text": "hsl(var(--foreground))",
          "--info-border": "hsl(var(--primary) / 0.3)",
          "--border-radius": "var(--radius)",
        } as React.CSSProperties
      }
      {...props}
    />
  );
};

export { Toaster };
