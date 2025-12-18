<script lang="ts">
    import { onMount, onDestroy } from "svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { openUrl } from "@tauri-apps/plugin-opener";
    import { fade } from "svelte/transition";
    import { listen } from "@tauri-apps/api/event";

    import { Button } from "$lib/components/ui/button/index";
    import * as Field from "$lib/components/ui/field/index";
    import { Input } from "$lib/components/ui/input/index";
    import * as Alert from "$lib/components/ui/alert/index";
    import { Skeleton } from "$lib/components/ui/skeleton/index";
    import * as Avatar from "$lib/components/ui/avatar";

    // Icons
    import CopyIcon from "@lucide/svelte/icons/copy";
    import CheckIcon from "@lucide/svelte/icons/check";
    import Loader2Icon from "@lucide/svelte/icons/loader-2";
    import ExternalLinkIcon from "@lucide/svelte/icons/external-link";
    import PartyPopperIcon from "@lucide/svelte/icons/party-popper";

    // --- State ---
    let step = $state<1 | 2 | 3 | 4 | 5>(1);
    let isLoading = $state(true);
    let isSubmitting = $state(false);
    let errorMessage = $state("");

    let clientId = $state("");
    let broadcasterLogin = $state("");

    let authUrl = $state("");
    let isCopied = $state(false);

    type ChatLog = {
        user: string;
        message: string;
        timestamp: string;
    };

    let chatLogs = $state<ChatLog[]>([]);
    let unlisten: (() => void) | undefined;

    // --- NEW: Helper to start listening (used by join and restore) ---
    async function startChatListener() {
        if (unlisten) unlisten(); // Clear existing if any

        unlisten = await listen<ChatLog>("chat-event", (event) => {
            chatLogs = [...chatLogs, event.payload].slice(-50);
            scrollToBottom();
        });
    }

    onMount(async () => {
        try {
            const isValid = await invoke<boolean>("check_auth_status");
            if (isValid) {
                // --- NEW: Check if we are already in a channel ---
                const alreadyActive = await invoke<boolean>("is_in_channel");

                if (alreadyActive) {
                    await startChatListener(); // Re-attach the listener
                    step = 5;
                } else {
                    step = 3;
                }
            }
        } catch (err) {
            console.error("Failed to check auth/channel status:", err);
        } finally {
            isLoading = false;
        }
    });

    onDestroy(() => {
        if (unlisten) unlisten();
    });

    // --- Actions ---

    async function handleCredentialsSubmit() {
        isSubmitting = true;
        errorMessage = "";

        try {
            const url = await invoke<string>("get_token", {
                clientId: clientId,
            });

            authUrl = url;
            step = 2;

            try {
                await openUrl(authUrl);
            } catch (e) {
                console.warn("Auto-open failed, user must click manually", e);
            }

            beginPollingForToken();
        } catch (error) {
            console.error("Error starting flow:", error);
            errorMessage = String(error);
        } finally {
            isSubmitting = false;
        }
    }

    async function beginPollingForToken() {
        try {
            await invoke("wait_for_token");
            step = 3;
        } catch (error) {
            console.error("Polling error:", error);
            errorMessage =
                "Authorization timed out or failed. Please try again.";
            step = 1;
        }
    }

    async function copyToClipboard(text: string) {
        try {
            await navigator.clipboard.writeText(text);
            isCopied = true;
            setTimeout(() => (isCopied = false), 2000);
        } catch (err) {
            console.error("Failed to copy:", err);
        }
    }

    async function handleJoinChannel() {
        isSubmitting = true;
        errorMessage = "";
        try {
            // Use the helper
            await startChatListener();

            await invoke("join_channel", {
                broadcasterLogin: broadcasterLogin,
            });

            step = 5;
        } catch (error) {
            console.error("Failed to join:", error);
            errorMessage = String(error);
        } finally {
            isSubmitting = false;
        }
    }

    async function handleDisconnect() {
        try {
            await invoke("leave_channel");
        } catch (error) {
            console.error("Error disconnecting:", error);
        } finally {
            step = 4;
            chatLogs = [];
            if (unlisten) {
                unlisten();
                unlisten = undefined;
            }
        }
    }

    let scrollContainer: HTMLDivElement | null;
    function scrollToBottom() {
        if (scrollContainer) {
            setTimeout(() => {
                scrollContainer!.scrollTop = scrollContainer!.scrollHeight;
            }, 0);
        }
    }

    function resetFlow() {
        step = 1;
        errorMessage = "";
        clientId = "";
    }
</script>

<div class="flex w-full max-w-auto flex-col p-6">
    {#if errorMessage}
        <div class="mb-6" transition:fade>
            <Alert.Root variant="destructive">
                <Alert.Title>Error</Alert.Title>
                <Alert.Description>{errorMessage}</Alert.Description>
            </Alert.Root>
        </div>
    {/if}

    {#if isLoading}
        <div class="flex flex-col gap-6 animate-pulse" transition:fade>
            <div class="space-y-2">
                <div class="h-5 w-32 rounded-md bg-muted"></div>
                <div class="h-4 w-full rounded-md bg-muted/50"></div>
                <div class="h-4 w-2/3 rounded-md bg-muted/50"></div>
            </div>
            <div class="space-y-4">
                <div class="space-y-2">
                    <div class="h-4 w-16 rounded-md bg-muted"></div>
                    <div class="h-10 w-full rounded-md bg-muted"></div>
                </div>
                <div class="space-y-2">
                    <div class="h-4 w-20 rounded-md bg-muted"></div>
                    <div class="h-10 w-full rounded-md bg-muted"></div>
                </div>
            </div>
            <div class="h-px w-full bg-muted"></div>
            <div class="flex justify-end">
                <div class="h-10 w-20 rounded-md bg-muted"></div>
            </div>
        </div>
    {:else if step === 1}
        <form
            class="flex flex-col gap-6"
            onsubmit={(e) => {
                e.preventDefault();
                handleCredentialsSubmit();
            }}
            transition:fade
        >
            <Field.Group>
                <Field.Set>
                    <Field.Legend>Bot Credentials</Field.Legend>
                    <Field.Description>
                        Enter your Twitch Application credentials to begin.
                        <a
                            href="https://dev.twitch.tv/console"
                            target="_blank"
                            class="text-primary hover:underline"
                            onclick={async (e) => {
                                e.preventDefault();
                                await openUrl("https://dev.twitch.tv/console");
                            }}
                        >
                            Open Developer Console
                        </a>
                    </Field.Description>

                    <Field.Group class="gap-4">
                        <Field.Field>
                            <Field.Label for="client-id">Client ID</Field.Label>
                            <Input
                                id="client-id"
                                placeholder="gp762nuuoqcoxypju8c569v9xflm4r"
                                bind:value={clientId}
                                required
                                disabled={isSubmitting}
                            />
                        </Field.Field>
                    </Field.Group>
                </Field.Set>
            </Field.Group>

            <Field.Separator />

            <div class="flex justify-end">
                <Button type="submit" disabled={isSubmitting}>
                    {#if isSubmitting}
                        <Loader2Icon class="mr-2 h-4 w-4 animate-spin" />
                        Connecting...
                    {:else}
                        Next
                    {/if}
                </Button>
            </div>
        </form>
    {:else if step === 2}
        <div class="flex flex-col gap-6" transition:fade>
            <Field.Group>
                <Field.Set>
                    <Field.Legend>Authorize Application</Field.Legend>
                    <Field.Description>
                        We've opened a browser window. Please approve the bot to
                        continue.
                    </Field.Description>

                    <div class="py-4">
                        <Button
                            class="w-full"
                            size="lg"
                            variant="outline"
                            onclick={async () => await openUrl(authUrl)}
                        >
                            <ExternalLinkIcon class="mr-2 h-4 w-4" />
                            Click here if browser didn't open
                        </Button>
                    </div>

                    <Field.Field>
                        <Field.Label>Authorization URL</Field.Label>
                        <div class="flex items-center gap-2">
                            <Input
                                readonly
                                value={authUrl}
                                class="font-mono text-xs text-muted-foreground"
                            />
                            <Button
                                variant="secondary"
                                size="icon"
                                class="shrink-0"
                                onclick={() => copyToClipboard(authUrl)}
                                title="Copy URL"
                            >
                                {#if isCopied}
                                    <CheckIcon class="h-4 w-4 text-green-500" />
                                {:else}
                                    <CopyIcon class="h-4 w-4" />
                                {/if}
                            </Button>
                        </div>
                    </Field.Field>
                </Field.Set>
            </Field.Group>

            <div
                class="flex items-center justify-center gap-2 rounded-md bg-muted/50 p-4 text-sm text-muted-foreground"
            >
                <Loader2Icon class="h-4 w-4 animate-spin" />
                Waiting for authorization...
            </div>

            <Button variant="ghost" onclick={() => (step = 1)}>Cancel</Button>
        </div>
    {:else if step === 3}
        <div
            class="flex flex-col items-center justify-center gap-6 py-8 text-center"
            transition:fade
        >
            <div
                class="flex h-16 w-16 items-center justify-center rounded-full bg-green-100 dark:bg-green-900"
            >
                <PartyPopperIcon
                    class="h-8 w-8 text-green-600 dark:text-green-400"
                />
            </div>

            <div class="space-y-2">
                <h3 class="text-xl font-semibold">Bot Connected!</h3>
                <p class="text-sm text-muted-foreground">
                    Your credentials have been saved and the token is secure.
                </p>
            </div>

            <div class="flex flex-col gap-2 w-full">
                <Button
                    class="w-full"
                    onclick={() => {
                        step = 4;
                    }}
                >
                    Connect to a stream
                </Button>
                <Button
                    variant="link"
                    class="text-xs text-muted-foreground"
                    onclick={resetFlow}
                >
                    Configure different account
                </Button>
            </div>
        </div>
    {:else if step == 4}
        <form
            class="flex flex-col gap-6"
            onsubmit={(e) => {
                e.preventDefault();
                handleJoinChannel();
            }}
            transition:fade
        >
            <Field.Group>
                <Field.Set>
                    <Field.Legend>Connect to a stream</Field.Legend>
                    <Field.Description>
                        Enter the Twitch username of the channel you want to
                        join.
                    </Field.Description>

                    <Field.Group class="gap-4">
                        <Field.Field>
                            <Field.Label for="broadcaster-login"
                                >Broadcaster Login</Field.Label
                            >
                            <Input
                                id="broadcasterLogin"
                                placeholder="e.g. Rhiel"
                                bind:value={broadcasterLogin}
                                required
                                disabled={isSubmitting}
                            />
                        </Field.Field>
                    </Field.Group>
                </Field.Set>
            </Field.Group>

            <Field.Separator />

            <div class="flex justify-end">
                <Button type="submit" disabled={isSubmitting}>
                    {#if isSubmitting}
                        <Loader2Icon class="mr-2 h-4 w-4 animate-spin" />
                        Connecting...
                    {:else}
                        Connect
                    {/if}
                </Button>
            </div>
        </form>
    {:else if step == 5}
        <div class="flex flex-col h-[500px] w-full" transition:fade>
            <div class="flex items-center justify-between border-b pb-4 mb-4">
                <div class="flex flex-col">
                    <h3 class="font-semibold">
                        Connected to {broadcasterLogin}
                    </h3>
                    <span
                        class="text-xs text-muted-foreground flex items-center gap-1"
                    >
                        <span
                            class="h-2 w-2 rounded-full bg-green-500 animate-pulse"
                        ></span>
                        Live EventSub
                    </span>
                </div>
                <Button variant="outline" size="sm" onclick={handleDisconnect}>
                    Disconnect
                </Button>
            </div>

            <div
                class="flex-1 overflow-y-auto space-y-4 pr-2 scrollbar-thin"
                bind:this={scrollContainer}
            >
                {#if chatLogs.length === 0}
                    <div
                        class="flex h-full items-center justify-center text-muted-foreground text-sm italic"
                    >
                        Waiting for messages...
                    </div>
                {/if}

                {#each chatLogs as log}
                    <div class="flex items-start gap-3 text-sm">
                        <Avatar.Root class="h-8 w-8">
                            <Avatar.Fallback
                                >{log.user
                                    .substring(0, 2)
                                    .toUpperCase()}</Avatar.Fallback
                            >
                        </Avatar.Root>
                        <div class="flex flex-col">
                            <div class="flex items-center gap-2">
                                <span class="font-bold">{log.user}</span>
                                <span class="text-[10px] text-muted-foreground"
                                    >{log.timestamp}</span
                                >
                            </div>
                            <p class="text-foreground/90 leading-relaxed">
                                {log.message}
                            </p>
                        </div>
                    </div>
                {/each}
            </div>
        </div>
    {/if}
</div>
