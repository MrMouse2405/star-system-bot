<script lang="ts">
	import "../app.css";
	import * as Breadcrumb from "$lib/components/ui/breadcrumb/index.js";
	import { Separator } from "$lib/components/ui/separator/index.js";
	import * as Sidebar from "$lib/components/ui/sidebar/index";
	import AppSidebar from "./app-sidebar.svelte";

	import { ModeWatcher } from "mode-watcher";

	import { page } from "$app/state";

	let { children } = $props();

	function get_page_name(): string {
		switch (page.url.pathname) {
			case "/":
				return "Star System Bot";
			case "/bot":
				return "Bot";
			case "/test":
				return "Test";
			case "/settings":
				return "Settings";
			default:
				return "<error>";
		}
	}
</script>

<Sidebar.Provider>
	<AppSidebar />
	<Sidebar.Inset class="flex flex-col h-screen">
		<header class="flex h-16 shrink-0 items-center gap-2">
			<div class="flex items-center gap-2 px-4">
				<Sidebar.Trigger class="-ms-1" />
				<Separator
					orientation="vertical"
					class="me-2 data-[orientation=vertical]:h-4"
				/>
				<Breadcrumb.Root>
					<Breadcrumb.List>
						<Breadcrumb.Item>
							<Breadcrumb.Page>{get_page_name()}</Breadcrumb.Page>
						</Breadcrumb.Item>
					</Breadcrumb.List>
				</Breadcrumb.Root>
			</div>
		</header>
		<main class="flex-1 min-h-0">
			<ModeWatcher />
			{@render children?.()}
		</main>
	</Sidebar.Inset>
</Sidebar.Provider>
