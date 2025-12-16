<script lang="ts">
    import { cn } from "$lib/utils";
    import type { Message } from "$lib/stores/chat";

    export let message: Message;

    const isBot = message.sender === "bot";
    const timeString = message.timestamp.toLocaleTimeString([], {
        hour: "2-digit",
        minute: "2-digit",
    });
</script>

<div class={cn("flex w-full", isBot ? "justify-start" : "justify-end")}>
    <div
        class={cn(
            "max-w-[75%] rounded-lg p-3 shadow-md",
            isBot
                ? "bg-muted text-muted-foreground rounded-tl-none"
                : "bg-primary text-primary-foreground rounded-br-none",
        )}
    >
        <p class="text-sm">
            {message.content}
        </p>

        <p
            class={cn(
                "mt-1 text-xs opacity-50",
                isBot ? "text-right" : "text-left",
            )}
        >
            {timeString}
        </p>
    </div>
</div>
