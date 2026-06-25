<script lang="ts">
  import {
    DataTable,
    DataTableSkeleton,
    Pagination,
  } from "carbon-components-svelte";
  import apiClient from "../../lib/api";

  // Table header
  const headers = [
    { key: "id", value: "Id" },
    { key: "username", value: "Username" },
    { key: "enabled", value: "Enabled" },
    { key: "created_at", value: "Created at" },
  ] as const;

  // Pagination value
  let pageSize = 10;
  let page = 1;

  // Data from backend
  let rows: UserItem[] = [];
  let totalItems: number = 0;
  let loading: boolean = true;

  interface ReadAllNormalUserResponse {
    total: number;
    items: UserItem[];
  }

  interface UserItem {
    id: number;
    username: string;
    enabled: boolean;
    created_at: string;
  }

  // Get data from backend
  async function fetchData(currentPage: number, currentPageSize: number) {
    loading = true;
    try {
      const response = await apiClient.get<ReadAllNormalUserResponse>(
        `/mgmt/read_all_normal_user?page_index=${currentPage - 1}&page_size=${currentPageSize}`,
      );
      rows = response.data.items;
      totalItems = response.data.total;
    } catch (error) {
      console.error("Raad all users failed:", error);
    } finally {
      loading = false;
    }
  }

  $: fetchData(page, pageSize);
</script>

{#if loading}
  <DataTableSkeleton {headers} rows={pageSize} />
{:else}
  <DataTable title="Users" {headers} {rows} />
{/if}

<Pagination {totalItems} pageSizes={[5, 10, 20, 50]} bind:pageSize bind:page />
