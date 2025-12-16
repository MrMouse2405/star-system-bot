<script lang="ts">
    import * as Chat from "$lib/components/ui/chat";
    import * as Avatar from "$lib/components/ui/avatar";
    import { Button } from "$lib/components/ui/button";
    import { Input } from "$lib/components/ui/input";
    import { Badge } from "$lib/components/ui/badge";
    import { Alert, AlertDescription } from "$lib/components/ui/alert";
    import SendIcon from "@lucide/svelte/icons/send";
    import AlertCircleIcon from "@lucide/svelte/icons/alert-circle";
    import LanguagesIcon from "@lucide/svelte/icons/languages";
    import { invoke } from "@tauri-apps/api/core";
    import { UseAutoScroll } from "$lib/hooks/use-auto-scroll.svelte";

    interface TranslationResponse {
        language: string;
        translation: string;
    }

    type Message = {
        id: string;
        content: string;
        variant: "sent" | "received";
        timestamp: string;
        language?: string;
        isError?: boolean;
    };

    const autoScroll = new UseAutoScroll();

    const formatTime = (date: Date) => {
        return date.toLocaleTimeString("en-US", {
            hour: "numeric",
            minute: "2-digit",
            hour12: true,
        });
    };

    let userMessage = $state("");
    let messages = $state<Message[]>([]);
    let isTyping = $state(false);

    async function handleSendMessage(e: Event) {
        e.preventDefault();
        const trimmedMessage = userMessage.trim();
        if (!trimmedMessage) return;

        messages.push({
            id: crypto.randomUUID(),
            content: trimmedMessage,
            variant: "sent",
            timestamp: formatTime(new Date()),
        });

        userMessage = "";
        isTyping = true;

        try {
            const result = (await invoke("translate", {
                text: trimmedMessage,
            })) as TranslationResponse;

            messages.push({
                id: crypto.randomUUID(),
                content: String(result.translation),
                variant: "received",
                timestamp: formatTime(new Date()),
                language: result.language,
            });

            autoScroll.scrollToBottom();
        } catch (error) {
            console.error("Translation error:", error);
            messages.push({
                id: crypto.randomUUID(),
                content:
                    "We encountered an error while translating your message. This could be due to a network issue or service unavailability. Please try again in a moment.",
                variant: "received",
                timestamp: formatTime(new Date()),
                isError: true,
            });

            autoScroll.scrollToBottom();
        } finally {
            isTyping = false;
        }
    }
</script>

<div class="flex flex-col h-full w-full">
    <!-- Header - stays at top -->
    <header class="flex shrink-0 items-center gap-3 border-b bg-background p-4">
        <Avatar.Root>
            <Avatar.Fallback>TR</Avatar.Fallback>
        </Avatar.Root>
        <div class="flex flex-col">
            <span class="text-sm font-medium">Star System Bot</span>
            <span class="text-xs text-muted-foreground">
                This is not a LLM! It only translates!
            </span>
        </div>
    </header>

    <div
        class="flex-1 min-h-0 overflow-y-auto scrollbar-hide"
        bind:this={autoScroll.ref}
    >
        <Chat.List class="h-full">
            {#each messages as msg (msg.id)}
                {#if msg.isError}
                    <!-- Error Message -->
                    <div class="px-4 py-2">
                        <Alert
                            variant="destructive"
                            class="border-destructive/50"
                        >
                            <AlertCircleIcon class="h-4 w-4" />
                            <AlertDescription class="ml-2">
                                {msg.content}
                            </AlertDescription>
                        </Alert>
                        <div
                            class="text-xs text-muted-foreground mt-1 text-end"
                        >
                            {msg.timestamp}
                        </div>
                    </div>
                {:else}
                    <!-- Regular Message -->
                    <Chat.Bubble variant={msg.variant}>
                        <Chat.BubbleAvatar>
                            <Chat.BubbleAvatarFallback>
                                {msg.variant === "sent" ? "You" : "TR"}
                            </Chat.BubbleAvatarFallback>
                        </Chat.BubbleAvatar>
                        <Chat.BubbleMessage class="flex flex-col gap-2">
                            {#if msg.language && msg.variant === "received"}
                                <div class="flex items-center gap-1.5">
                                    <LanguagesIcon
                                        class="h-3.5 w-3.5 text-muted-foreground"
                                    />
                                    <Badge
                                        variant="secondary"
                                        class="text-xs font-normal"
                                    >
                                        Translated from {msg.language}
                                    </Badge>
                                </div>
                            {/if}
                            <p>{msg.content}</p>
                            <div
                                class="w-full text-xs group-data-[variant='sent']/chat-bubble:text-end text-muted-foreground"
                            >
                                {msg.timestamp}
                            </div>
                        </Chat.BubbleMessage>
                    </Chat.Bubble>
                {/if}
            {/each}
            {#if isTyping}
                <Chat.Bubble variant="received">
                    <Chat.BubbleAvatar>
                        <Chat.BubbleAvatarFallback>TR</Chat.BubbleAvatarFallback
                        >
                    </Chat.BubbleAvatar>
                    <Chat.BubbleMessage typing />
                </Chat.Bubble>
            {/if}
        </Chat.List>
    </div>

    <!-- Footer - stays at bottom -->
    <footer class="shrink-0 border-t bg-background">
        <form onsubmit={handleSendMessage} class="flex items-center gap-2 p-4">
            <Input
                bind:value={userMessage}
                class="flex-1"
                placeholder="Type your message..."
                required
            />
            <Button
                type="submit"
                size="icon"
                class="shrink-0"
                disabled={!userMessage.trim()}
            >
                <SendIcon class="h-5 w-5" />
            </Button>
        </form>
    </footer>
</div>
