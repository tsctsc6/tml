<script lang="ts">
  import {
    Button,
    DataTable,
    DataTableSkeleton,
    Modal,
    Pagination,
    TextInput,
    Toolbar,
    ToolbarContent,
    NotificationQueue,
  } from "carbon-components-svelte";
  import TrashCan from "carbon-icons-svelte/lib/TrashCan.svelte";
  import Add from "carbon-icons-svelte/lib/Add.svelte";
  import CloudDownload from "carbon-icons-svelte/lib/CloudDownload.svelte";
  import { apiClientExt } from "../../lib/api";

  let queue: NotificationQueue;

  /// Read all
  interface ReadAllStorageResponse {
    total: number;
    items: StorageItem[];
  }

  interface StorageItem {
    id: number;
    name: string;
    path: string;
    created_at: string;
  }

  // Table header
  const headers = [
    { key: "id", value: "Id" },
    { key: "name", value: "Name" },
    { key: "path", value: "Path" },
    { key: "delete", value: "Delete", empty: true },
  ] as const;

  // Pagination value
  let pageSize = 10;
  let page = 1;

  // Data from backend
  let rows: StorageItem[] = [];
  let totalItems: number = 0;
  let loading: boolean = true;

  // Get data from backend
  async function fetchData(currentPage: number, currentPageSize: number) {
    loading = true;
    try {
      const response = await apiClientExt.get<ReadAllStorageResponse>(
        `/mgmt/read_all_storage?page_index=${currentPage - 1}&page_size=${currentPageSize}`,
      );
      if (!response.success || !response.data) {
        throw new Error(response.message ?? "");
      }
      rows = response.data.items;
      totalItems = response.data.total;
    } catch (error: any) {
      queue.add({
        kind: "error",
        title: "Error",
        subtitle: error.toString(),
        timeout: 3000,
      });
    } finally {
      loading = false;
    }
  }

  $: fetchData(page, pageSize);

  /// Update
  interface UpdateStorageRequest {
    id: number;
    name: string;
    path: string;
  }

  interface UpdateStorageResponse {}

  let isEditingModalOpen = false;
  let isEditingSubmitting = false;
  // Copy a data, in case user not save
  let editingItem: UpdateStorageRequest = {
    id: 0,
    name: "",
    path: "",
  };

  // Row click event
  function handleRowClick(event: CustomEvent<{ row: StorageItem }>) {
    const clickedRow = event.detail.row;
    editingItem.id = clickedRow.id;
    editingItem.name = clickedRow.name;
    editingItem.path = clickedRow.path;
    isEditingModalOpen = true;
  }

  // Submit changes to backend
  async function handleSaveChanges() {
    isEditingSubmitting = true;
    try {
      const updateStorageRequest: UpdateStorageRequest = {
        id: editingItem.id,
        name: editingItem.name,
        path: editingItem.path,
      };
      const response = await apiClientExt.post<UpdateStorageResponse>(
        "/mgmt/update_storage",
        updateStorageRequest,
      );
      if (!response.success || !response.data) {
        throw new Error(response.message ?? "");
      }
      isEditingModalOpen = false;

      // Update row in datatable
      rows = rows.map((item) => {
        if (item.id === editingItem.id) {
          item.name = editingItem.name;
          item.path = editingItem.path;
        }
        return item;
      });
    } catch (error: any) {
      queue.add({
        kind: "error",
        title: "Error",
        subtitle: error.toString(),
        timeout: 3000,
      });
    } finally {
      isEditingSubmitting = false;
    }
  }

  /// Delete
  interface DeleteStorageRequest {
    id: number;
  }

  interface DeleteStorageResponse {}

  let isDeleteModalOpen = false;
  let itemToDelete: StorageItem | null = null;
  let isDeleting = false;

  function triggerDelete(event: Event, item: StorageItem) {
    event.stopPropagation();
    itemToDelete = item;
    isDeleteModalOpen = true;
  }

  async function confirmDelete() {
    if (!itemToDelete) return;
    isDeleting = true;

    try {
      const deleteStorageRequest: DeleteStorageRequest = {
        id: itemToDelete.id,
      };
      const response = await apiClientExt.post<DeleteStorageResponse>(
        "/mgmt/delete_storage",
        deleteStorageRequest,
      );
      if (!response.success || !response.data) {
        throw new Error(response.message ?? "");
      }
      rows = rows.filter((item) => item.id !== itemToDelete!.id);
      isDeleteModalOpen = false;
    } catch (error: any) {
      queue.add({
        kind: "error",
        title: "Error",
        subtitle: error.toString(),
        timeout: 3000,
      });
    } finally {
      isDeleting = false;
      itemToDelete = null;
    }
  }

  /// Create
  interface CreateStorageRequest {
    name: string;
    path: string;
  }

  interface CreateStorageResponse {}

  let isCreateModalOpen = false;
  let itemToCreate: CreateStorageRequest = {
    name: "",
    path: "",
  };
  let isCreating = false;

  function triggerCreate() {
    itemToCreate = {
      name: "",
      path: "",
    };
    isCreateModalOpen = true;
  }

  async function confirmCreate() {
    isCreating = true;
    try {
      const response = await apiClientExt.post<CreateStorageResponse>(
        "/mgmt/create_storage",
        itemToCreate,
      );
      if (!response.success || !response.data) {
        throw new Error(response.message ?? "");
      }
      fetchData(page, pageSize);
    } catch (error: any) {
      queue.add({
        kind: "error",
        title: "Error",
        subtitle: error.toString(),
        timeout: 3000,
      });
    } finally {
      isCreating = false;
      isCreateModalOpen = false;
    }
  }
</script>

<NotificationQueue bind:this={queue} />

{#if loading}
  <DataTableSkeleton {headers} rows={pageSize} />
{:else}
  <DataTable title="Storages" {headers} {rows} on:click:row={handleRowClick}>
    <Toolbar>
      <ToolbarContent>
        <Button
          icon={CloudDownload}
          iconDescription="Refresh"
          on:click={() => fetchData(page, pageSize)}
        />
        <Button icon={Add} iconDescription="Add" on:click={triggerCreate}
        ></Button>
      </ToolbarContent>
    </Toolbar>
    <svelte:fragment slot="cell" let:cell let:row>
      {#if cell.key === "delete"}
        <Button
          kind="danger-tertiary"
          size="small"
          iconDescription="Delete"
          icon={TrashCan}
          on:click={(e) => triggerDelete(e, row as StorageItem)}
        />
      {:else}
        {cell.value}
      {/if}
    </svelte:fragment>
  </DataTable>
{/if}

<Pagination {totalItems} pageSizes={[5, 10, 20, 50]} bind:pageSize bind:page />

<Modal
  bind:open={isEditingModalOpen}
  modalHeading="Edit Storage"
  primaryButtonText={isEditingSubmitting ? "Saving" : "Save"}
  secondaryButtonText="Cancel"
  primaryButtonDisabled={isEditingSubmitting}
  on:click:button--primary={handleSaveChanges}
  on:click:button--secondary={() => (isEditingModalOpen = false)}
  on:close={() => (isEditingModalOpen = false)}
>
  <div class="edit-form">
    <TextInput
      labelText="Id"
      bind:value={editingItem.id}
      placeholder="Id"
      readonly
    />
  </div>
  <div class="edit-form">
    <TextInput
      labelText="Name"
      bind:value={editingItem.name}
      placeholder="Name"
    />
  </div>
  <div class="edit-form">
    <TextInput
      labelText="Path"
      bind:value={editingItem.path}
      placeholder="Path"
    />
  </div>
</Modal>

<Modal
  danger
  bind:open={isDeleteModalOpen}
  modalHeading="Are you sure to delete?"
  primaryButtonText={isDeleting ? "Deleting..." : "Delete"}
  secondaryButtonText="Cancel"
  primaryButtonDisabled={isDeleting}
  on:click:button--primary={confirmDelete}
  on:click:button--secondary={() => (isDeleteModalOpen = false)}
  on:close={() => (isDeleteModalOpen = false)}
>
  <p>
    Are you sure to delete storage <strong>{itemToDelete?.id}</strong> ?
  </p>
  <p>All relevant information will be lost.</p>
</Modal>

<Modal
  bind:open={isCreateModalOpen}
  modalHeading="Create Storage"
  primaryButtonText={isCreating ? "Creating" : "Create"}
  secondaryButtonText="Cancel"
  primaryButtonDisabled={isCreating}
  on:click:button--primary={confirmCreate}
  on:click:button--secondary={() => (isCreateModalOpen = false)}
  on:close={() => (isCreateModalOpen = false)}
>
  <div class="edit-form">
    <TextInput
      labelText="Name"
      bind:value={itemToCreate.name}
      placeholder="Name"
    />
  </div>
  <div class="edit-form">
    <TextInput
      labelText="Path"
      bind:value={itemToCreate.path}
      placeholder="Path"
    />
  </div>
</Modal>

<style>
  .edit-form :global(.bx--form-item) {
    margin-bottom: 1.25rem;
  }
</style>
