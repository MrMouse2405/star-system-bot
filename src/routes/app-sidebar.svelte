<script lang="ts">
    import SettingsIcon from "@lucide/svelte/icons/settings";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import BotIcon from "@lucide/svelte/icons/bot";
    import BotMessageSquareIcon from "@lucide/svelte/icons/bot-message-square";

    // Menu items.
    const items = [
        {
            title: "Bot",
            url: "/bot",
            icon: BotIcon,
        },
        {
            title: "Test",
            url: "/test",
            icon: BotMessageSquareIcon,
        },
        {
            title: "Settings",
            url: "/settings",
            icon: SettingsIcon,
        },
    ];
</script>

<Sidebar.Root>
    <Sidebar.Content>
        <Sidebar.Group>
            <Sidebar.GroupLabel>Application</Sidebar.GroupLabel>
            <Sidebar.GroupContent>
                <Sidebar.Menu>
                    {#each items as item (item.title)}
                        <Sidebar.MenuItem>
                            <Sidebar.MenuButton>
                                {#snippet child({ props })}
                                    <a href={item.url} {...props}>
                                        <item.icon />
                                        <span>{item.title}</span>
                                    </a>
                                {/snippet}
                            </Sidebar.MenuButton>
                        </Sidebar.MenuItem>
                    {/each}
                </Sidebar.Menu>
            </Sidebar.GroupContent>
        </Sidebar.Group>
    </Sidebar.Content>
</Sidebar.Root>
