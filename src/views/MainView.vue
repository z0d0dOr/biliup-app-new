<template>
    <div class="main-view">
        <!-- 拖拽覆盖层 -->
        <div v-if="isDragOver" class="drag-overlay">
            <div class="drag-content">
                <el-icon class="drag-icon"><upload-filled /></el-icon>
                <h3>拖拽视频文件到此处</h3>
                <p>支持 MP4、AVI、MOV、MKV、WMV、FLV、M4V、WEBM 格式</p>
                <p v-if="!selectedUser || !currentTemplateName" class="warning-text">
                    请先选择用户和模板
                </p>
            </div>
        </div>

        <!-- 顶部导航栏 -->
        <el-header class="header">
            <div class="header-content">
                <div class="header-left">
                    <h2 class="app-title">Biliup APP</h2>
                    <div class="app-version">(v{{ currentVer }})</div>
                </div>
                                <div class="header-center">
                    <el-button
                        type="warning"
                        size="small"
                        @click="openTranslationConfigDialog"
                        title="翻译配置"
                    >
                        翻译配置
                    </el-button>
                    <el-button type="info" size="small" @click="exportLogs" title="导出日志">
                        导出日志
                    </el-button>
                    <el-button type="primary" size="small" @click="checkUpdate" title="检查更新">
                        检查更新
                    </el-button>
                    <el-button type="warning" size="small" plain @click="switchToPublishTimeEditMode">
                        修改发布时间
                    </el-button>
                    <el-button type="success" size="small" plain @click="switchToUploadMode">
                        上传视频
                    </el-button>
                </div>
                <div class="header-right">
                    <!-- 上传队列下拉框 -->
                    <UploadQueue />

                    <!-- 全局设置按钮 -->
                    <el-button
                        type="info"
                        size="small"
                        circle
                        @click="showGlobalConfigDialog = true"
                        title="全局设置"
                        class="global-config-btn"
                    >
                        <el-icon><setting /></el-icon>
                    </el-button>

                    <!-- 用户列表下拉框 -->
                    <UserList
                        @show-login="showLoginDialog = true"
                        @user-logout="handleLogoutUser"
                    />
                </div>
            </div>
        </el-header>

        <el-container class="main-container">
            <!-- 用户模板侧边栏 -->
            <el-aside width="300px" class="sidebar">
                <div class="sidebar-header">
                    <h3></h3>
                    <div class="header-buttons">
                        <el-checkbox
                            v-model="highlightAutoSubmitting"
                            size="small"
                            class="highlight-checkbox"
                        >
                            <span class="highlight-checkbox-text"> 高亮显示<br />自动提交 </span>
                        </el-checkbox>
                        <el-button
                            type="success"
                            size="small"
                            @click="showLoginDialog = true"
                            :disabled="templateLoading"
                        >
                            <el-icon><user /></el-icon>
                            登录用户
                        </el-button>
                        <el-button
                            type="primary"
                            size="small"
                            @click="showNewTemplateDialog = true"
                            :disabled="!loginUsers.length || templateLoading"
                        >
                            <el-icon><plus /></el-icon>
                            新建模板
                        </el-button>
                    </div>
                </div>

                <div class="sidebar-content">
                    <div class="user-template-list">
                        <div
                            v-for="userTemplate in userTemplates"
                            :key="userTemplate.user.uid"
                            class="user-section"
                        >
                            <!-- 用户头部 -->
                            <div
                                class="user-header"
                                @click="handleUserExpansion(userTemplate.user.uid)"
                                :class="{ disabled: templateLoading }"
                            >
                                <el-avatar
                                    :src="`data:image/jpeg;base64,${userTemplate.user.avatar}`"
                                    :size="24"
                                    class="user-avatar"
                                >
                                    {{ userTemplate.user.username.charAt(0) }}
                                </el-avatar>
                                <span class="user-name">{{ userTemplate.user.username }}</span>
                                <el-icon
                                    class="config-icon"
                                    @click.stop="openUserConfig(userTemplate.user)"
                                    title="用户配置"
                                >
                                    <setting />
                                </el-icon>
                                <el-badge
                                    :value="userTemplate.templates.length"
                                    class="template-count-badge"
                                />
                                <el-icon
                                    class="expand-icon"
                                    :class="{ expanded: userTemplate.expanded }"
                                >
                                    <arrow-down />
                                </el-icon>
                            </div>

                            <!-- 模板列表 -->
                            <div class="template-list" v-show="userTemplate.expanded">
                                <div
                                    v-for="template in userTemplate.templates"
                                    :key="`${userTemplate.user.uid}-${template.name}`"
                                    class="template-item"
                                    :class="{
                                        active:
                                            selectedUser?.uid === userTemplate.user.uid &&
                                            currentTemplateName === template.name,
                                        'auto-submitting':
                                            highlightAutoSubmitting &&
                                            autoSubmittingRecord[
                                                getTemplateKey(userTemplate.user.uid, template.name)
                                            ],
                                        'auto-submitting-simple':
                                            !highlightAutoSubmitting &&
                                            autoSubmittingRecord[
                                                getTemplateKey(userTemplate.user.uid, template.name)
                                            ],
                                        'template-loading':
                                            templateLoading &&
                                            selectedUser?.uid === userTemplate.user.uid &&
                                            currentTemplateName === template.name,
                                        disabled: templateLoading
                                    }"
                                    @click="
                                        handleTemplateSelection(userTemplate.user, template.name)
                                    "
                                >
                                    <div class="template-main">
                                        <div class="template-name">
                                            {{ template.name }}
                                            <span
                                                v-show="
                                                    checkTemplateHasUnsavedChanges(
                                                        userTemplate.user.uid,
                                                        template.name
                                                    )
                                                "
                                                class="unsaved-indicator"
                                                title="有未保存的修改"
                                            ></span>
                                        </div>
                                        <div class="template-desc">
                                            {{ template.config.title || '无标题' }}
                                        </div>
                                    </div>
                                    <el-dropdown
                                        @command="
                                            (command: string) =>
                                                handleTemplateCommand(
                                                    command,
                                                    userTemplate.user,
                                                    template
                                                )
                                        "
                                        @click.stop
                                        trigger="click"
                                        :disabled="templateLoading"
                                    >
                                        <el-button link size="small" class="template-menu-btn">
                                            <el-icon><more-filled /></el-icon>
                                        </el-button>
                                        <template #dropdown>
                                            <el-dropdown-menu>
                                                <el-dropdown-item command="duplicate"
                                                    >复制</el-dropdown-item
                                                >
                                                <el-dropdown-item command="rename"
                                                    >重命名</el-dropdown-item
                                                >
                                                <el-dropdown-item command="delete" divided
                                                    >删除</el-dropdown-item
                                                >
                                            </el-dropdown-menu>
                                        </template>
                                    </el-dropdown>
                                </div>
                            </div>
                        </div>

                        <!-- 空状态 -->
                        <div v-if="userTemplates.length === 0" class="empty-users">
                            <el-empty description="暂无登录用户">
                                <el-button type="primary" @click="showLoginDialog = true">
                                    去登录
                                </el-button>
                            </el-empty>
                        </div>
                    </div>
                </div>
            </el-aside>

            <!-- 主要内容区域 -->
            <el-main class="main-content" v-if="selectedUser || currentForm || isPublishTimeEditMode">
                <div class="content-wrapper" ref="contentWrapperRef">
                    <div v-if="!selectedUser" class="no-selection">
                        <el-empty description="请选择用户和模板开始使用" />
                    </div>

                    <div v-else-if="isPublishTimeEditMode" class="publish-time-editor-shell">
                        <PublishTimeEditor :selected-user="selectedUser" />
                    </div>

                    <div v-else-if="!currentTemplateName" class="no-template">
                        <el-empty description="请选择模板或创建新模板">
                            <el-button type="primary" @click="showNewTemplateDialog = true">
                                新建模板
                            </el-button>
                        </el-empty>
                    </div>

                    <div v-else-if="currentForm" class="upload-form-container">
                        <div class="form-header">
                            <div class="template-name-container">
                                <h3 class="edit-bv-template-disaplay" v-if="currentTemplate?.aid">
                                    编辑稿件：
                                </h3>
                                <el-tooltip
                                    v-if="currentTemplate?.aid"
                                    content="刷新稿件数据"
                                    placement="top"
                                >
                                    <el-icon
                                        class="refresh-btn"
                                        @click.stop="
                                            reloadTemplateFromAV(
                                                selectedUser.uid,
                                                currentTemplate.aid
                                            )
                                        "
                                    >
                                        <refresh />
                                    </el-icon>
                                </el-tooltip>
                                <h3
                                    v-if="!isEditingTemplateName"
                                    @click="handleTemplateNameEdit"
                                    class="template-name-display"
                                    :class="{ disabled: templateLoading }"
                                    :title="
                                        templateLoading
                                            ? '模板加载中，无法编辑'
                                            : '点击编辑模板名称'
                                    "
                                >
                                    {{ currentTemplateName }}
                                    <el-icon class="edit-hint-icon"><edit /></el-icon>
                                </h3>
                                <el-input
                                    v-else
                                    ref="templateNameInputRef"
                                    v-model="editingTemplateName"
                                    @blur="saveTemplateName"
                                    @keyup.enter="saveTemplateName"
                                    @keyup.esc="cancelEditTemplateName"
                                    class="template-name-input"
                                    size="large"
                                    :disabled="templateLoading"
                                />
                            </div>
                            <div class="header-actions">
                                <el-button @click="resetTemplate" :disabled="templateLoading"
                                    >放弃更改</el-button
                                >
                                <el-button
                                    type="primary"
                                    @click="saveTemplate"
                                    :disabled="templateLoading"
                                    >保存</el-button
                                >
                                <el-button
                                    @click="
                                        handleTemplateCommand('delete', selectedUser, {
                                            name: currentTemplateName,
                                            config: currentTemplate
                                        })
                                    "
                                    @click.stop
                                    trigger="click"
                                    type="danger"
                                    :disabled="templateLoading"
                                    >删除</el-button
                                >
                            </div>
                        </div>

                        <el-form :model="currentForm" label-width="80px" class="upload-form">
                            <!-- 基本信息 -->
                            <el-card
                                class="form-section"
                                :class="{ collapsed: cardCollapsed.basic }"
                            >
                                <template #header>
                                    <div class="card-header" @click="toggleCardCollapsed('basic')">
                                        <span>基本信息</span>
                                        <div class="header-actions">
                                            <el-button
                                                type="danger"
                                                text
                                                size="small"
                                                @click.stop="clearCardContent('basic')"
                                                title="清空基本信息"
                                                :disabled="templateLoading"
                                            >
                                                <el-icon><delete /></el-icon>
                                            </el-button>
                                            <el-icon
                                                class="collapse-icon"
                                                :class="{ collapsed: cardCollapsed.basic }"
                                            >
                                                <arrow-down />
                                            </el-icon>
                                        </div>
                                    </div>
                                </template>

                                <el-collapse-transition>
                                    <div v-show="!cardCollapsed.basic" class="card-content">
                                        <el-form-item label="视频标题" required>
                                            <div class="title-input-row">
                                                <el-input
                                                    v-model="currentForm.title_prefix"
                                                    class="title-prefix-input"
                                                    placeholder="请输入标题前缀（可选）"
                                                    :disabled="templateLoading"
                                                />
                                                <el-input
                                                    v-model="currentForm.title"
                                                    class="title-main-input"
                                                    placeholder="请输入视频标题"
                                                    :disabled="templateLoading"
                                                />
                                                <el-button
                                                    class="title-translate-btn"
                                                    type="primary"
                                                    plain
                                                    size="small"
                                                    :loading="titleTranslating"
                                                    :disabled="templateLoading || !canTranslateTitle"
                                                    @click="handleManualTranslateTitle"
                                                >
                                                    翻译
                                                </el-button>
                                            </div>
                                        </el-form-item>

                                        <el-form-item label="封面">
                                            <div
                                                class="cover-uploader"
                                                action="#"
                                                @click="handleCoverSelection"
                                                v-loading="coverLoading"
                                                :class="{ disabled: templateLoading }"
                                            >
                                                <img
                                                    v-if="coverDisplayUrl && !coverLoading"
                                                    :src="coverDisplayUrl"
                                                    class="cover-image"
                                                />
                                                <el-icon
                                                    v-else-if="!coverLoading"
                                                    class="cover-uploader-icon"
                                                >
                                                    <plus />
                                                </el-icon>
                                            </div>
                                        </el-form-item>

                                        <el-form-item label="视频分区">
                                            <el-popover
                                                v-model:visible="categoryPopoverVisible"
                                                placement="bottom-start"
                                                :width="600"
                                                trigger="click"
                                                popper-class="category-popover"
                                            >
                                                <template #reference>
                                                    <el-button
                                                        class="category-trigger"
                                                        :type="
                                                            currentForm.tid ? 'primary' : 'default'
                                                        "
                                                        :disabled="templateLoading"
                                                    >
                                                        <span class="category-text">
                                                            <span v-if="selectedSubCategory">
                                                                {{ selectedCategory?.name }} >
                                                                {{ selectedSubCategory?.name }}
                                                            </span>
                                                            <span v-else class="placeholder"
                                                                >请选择分区</span
                                                            >
                                                        </span>
                                                        <el-icon class="arrow-icon">
                                                            <arrow-down />
                                                        </el-icon>
                                                    </el-button>
                                                </template>

                                                <div class="category-selector-panel">
                                                    <!-- 左侧主分区列表 -->
                                                    <div class="category-list">
                                                        <div
                                                            v-for="category in typeList"
                                                            :key="category.id"
                                                            class="category-item"
                                                            :class="{
                                                                active:
                                                                    selectedCategory?.id ===
                                                                    category.id
                                                            }"
                                                            @click="onCategoryChange(category.id)"
                                                        >
                                                            <span class="category-name">{{
                                                                category.name
                                                            }}</span>
                                                            <el-icon class="arrow-right">
                                                                <arrow-down
                                                                    style="
                                                                        transform: rotate(-90deg);
                                                                    "
                                                                />
                                                            </el-icon>
                                                        </div>
                                                    </div>

                                                    <!-- 右侧子分区列表 -->
                                                    <div class="subcategory-list">
                                                        <div
                                                            v-if="
                                                                selectedCategory &&
                                                                selectedCategory.children
                                                            "
                                                        >
                                                            <div
                                                                v-for="subCategory in selectedCategory.children"
                                                                :key="subCategory.id"
                                                                class="subcategory-item"
                                                                :class="{
                                                                    active:
                                                                        selectedSubCategory?.id ===
                                                                        subCategory.id
                                                                }"
                                                                @click="
                                                                    onSubCategoryChange(
                                                                        subCategory.id
                                                                    )
                                                                "
                                                                :title="
                                                                    subCategory.intro_original ||
                                                                    subCategory.desc
                                                                "
                                                            >
                                                                <div class="subcategory-content">
                                                                    <div class="subcategory-name">
                                                                        {{ subCategory.name }}
                                                                    </div>
                                                                    <div class="subcategory-desc">
                                                                        {{
                                                                            subCategory.desc !== ''
                                                                                ? subCategory.desc
                                                                                : subCategory.intro_original
                                                                        }}
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        </div>
                                                        <div v-else class="empty-subcategory">
                                                            <el-empty
                                                                description="请选择左侧主分区"
                                                                :image-size="60"
                                                            />
                                                        </div>
                                                    </div>
                                                </div>
                                            </el-popover>
                                        </el-form-item>

                                        <el-form-item label="版权声明">
                                            <el-radio-group
                                                v-model="currentForm.copyright"
                                                :disabled="templateLoading"
                                            >
                                                <el-radio :value="1">自制</el-radio>
                                                <el-radio :value="2">转载</el-radio>
                                            </el-radio-group>
                                        </el-form-item>

                                        <el-form-item
                                            label="转载来源"
                                            v-if="currentForm.copyright === 2"
                                        >
                                            <el-input
                                                v-model="currentForm.source"
                                                placeholder="请填写转载来源"
                                                :disabled="templateLoading"
                                            />
                                        </el-form-item>
                                    </div>
                                </el-collapse-transition>
                            </el-card>

                            <!-- 视频文件 -->
                            <el-card
                                class="form-section"
                                :class="{
                                    'drag-target': isDragOver,
                                    collapsed: cardCollapsed.videos
                                }"
                            >
                                <template #header>
                                    <div class="card-header">
                                        <div
                                            style="
                                                display: flex;
                                                align-items: center;
                                                gap: 12px;
                                                flex: 1;
                                            "
                                            @click="toggleCardCollapsed('videos')"
                                        >
                                            <span style="cursor: pointer">视频文件</span>
                                            <el-button
                                                type="success"
                                                size="small"
                                                @click.stop="checkVideoStatus"
                                                v-if="
                                                    currentForm.videos &&
                                                    currentForm.videos.length > 0 &&
                                                    currentTemplate?.aid
                                                "
                                                :disabled="templateLoading"
                                            >
                                                视频转码状态
                                            </el-button>
                                        </div>
                                        <div class="header-actions">
                                            <span v-if="isDragOver" class="drag-hint"
                                                >拖拽文件到此处添加</span
                                            >
                                            <el-icon
                                                class="collapse-icon"
                                                :class="{ collapsed: cardCollapsed.videos }"
                                                @click="toggleCardCollapsed('videos')"
                                            >
                                                <arrow-down />
                                            </el-icon>
                                        </div>
                                    </div>
                                </template>

                                <el-collapse-transition>
                                    <div v-show="!cardCollapsed.videos" class="card-content">
                                        <VideoList
                                            v-model:videos="videos"
                                            :is-drag-over="isDragOver"
                                            :uploading="uploading"
                                            :template-title="currentTemplateName"
                                            :disabled="templateLoading"
                                            @select-video="selectVideoWithTauri"
                                            @clear-all-videos="clearAllVideos"
                                            @remove-file="removeUploadedFile"
                                            @create-upload="createUpload"
                                            @add-videos-to-form="handleAddVideosToForm"
                                            @submit-template="handleSubmitTemplate"
                                            @videos-reversed="handleVideosReversed"
                                        />
                                    </div>
                                </el-collapse-transition>
                            </el-card>

                            <!-- 标签设置 -->
                            <el-card
                                class="form-section"
                                :class="{ collapsed: cardCollapsed.tags }"
                            >
                                <template #header>
                                    <div class="card-header" @click="toggleCardCollapsed('tags')">
                                        <span>标签设置</span>
                                        <div class="header-actions">
                                            <el-button
                                                type="danger"
                                                text
                                                size="small"
                                                @click.stop="clearCardContent('tags')"
                                                title="清空标签设置"
                                                :disabled="templateLoading"
                                            >
                                                <el-icon><delete /></el-icon>
                                            </el-button>
                                            <el-icon
                                                class="collapse-icon"
                                                :class="{ collapsed: cardCollapsed.tags }"
                                            >
                                                <arrow-down />
                                            </el-icon>
                                        </div>
                                    </div>
                                </template>

                                <el-collapse-transition>
                                    <div v-show="!cardCollapsed.tags" class="card-content">
                                        <el-form-item label="视频标签">
                                            <TagView
                                                ref="tagViewRef"
                                                v-model="tags"
                                                :disabled="templateLoading"
                                            />
                                        </el-form-item>

                                        <el-form-item v-if="!currentForm.aid" label="参与活动">
                                            <TopicView
                                                v-model="currentForm.mission_id"
                                                v-model:topic-id="currentForm.topic_id"
                                                :user-uid="selectedUser?.uid"
                                                mode="selector"
                                                :disabled="templateLoading"
                                            />
                                        </el-form-item>
                                    </div>
                                </el-collapse-transition>
                            </el-card>

                            <!-- 视频描述 -->
                            <el-card
                                class="form-section"
                                :class="{ collapsed: cardCollapsed.description }"
                            >
                                <template #header>
                                    <div
                                        class="card-header"
                                        @click="toggleCardCollapsed('description')"
                                    >
                                        <span>视频描述</span>
                                        <div class="header-actions">
                                            <el-button
                                                type="danger"
                                                text
                                                size="small"
                                                @click.stop="clearCardContent('description')"
                                                title="清空视频描述"
                                                :disabled="templateLoading"
                                            >
                                                <el-icon><delete /></el-icon>
                                            </el-button>
                                            <el-icon
                                                class="collapse-icon"
                                                :class="{ collapsed: cardCollapsed.description }"
                                            >
                                                <arrow-down />
                                            </el-icon>
                                        </div>
                                    </div>
                                </template>

                                <el-collapse-transition>
                                    <div v-show="!cardCollapsed.description" class="card-content">
                                        <el-form-item label="视频描述">
                                            <el-input
                                                v-model="currentForm.desc"
                                                type="textarea"
                                                :rows="6"
                                                placeholder="请输入视频简介"
                                                maxlength="2000"
                                                show-word-limit
                                                :disabled="templateLoading"
                                            />
                                        </el-form-item>

                                        <el-form-item label="空间动态">
                                            <el-input
                                                v-model="currentForm.dynamic"
                                                placeholder="请输入动态内容"
                                                maxlength="233"
                                                show-word-limit
                                                :disabled="templateLoading"
                                            />
                                        </el-form-item>
                                    </div>
                                </el-collapse-transition>
                            </el-card>

                            <!-- 高级设置 -->
                            <el-card
                                class="form-section"
                                :class="{ collapsed: cardCollapsed.advanced }"
                            >
                                <template #header>
                                    <div
                                        class="card-header"
                                        @click="toggleCardCollapsed('advanced')"
                                    >
                                        <span>高级设置</span>
                                        <div class="header-actions">
                                            <el-button
                                                type="danger"
                                                text
                                                size="small"
                                                @click.stop="clearCardContent('advanced')"
                                                title="清空高级设置"
                                                :disabled="templateLoading"
                                            >
                                                <el-icon><delete /></el-icon>
                                            </el-button>
                                            <el-icon
                                                class="collapse-icon"
                                                :class="{ collapsed: cardCollapsed.advanced }"
                                            >
                                                <arrow-down />
                                            </el-icon>
                                        </div>
                                    </div>
                                </template>

                                <el-collapse-transition>
                                    <div v-show="!cardCollapsed.advanced" class="card-content">
                                        <el-form-item label="开启水印">
                                            <div div class="checkbox-group">
                                                <el-checkbox
                                                    v-model="currentForm.watermark"
                                                    :true-value="1"
                                                    :false-value="0"
                                                    :disabled="templateLoading"
                                                >
                                                    开启 (本功能只对本次上传的视频生效)
                                                </el-checkbox>
                                            </div>
                                        </el-form-item>
                                        <el-form-item v-if="!currentForm.aid" label="定时发布">
                                            <el-date-picker
                                                v-model="dtimeDate"
                                                type="datetime"
                                                placeholder="选择发布时间"
                                                format="YYYY-MM-DD HH:mm:ss"
                                                :disabled="templateLoading"
                                                :disabled-date="
                                                    (date: Date) => {
                                                        const now = new Date()
                                                        const twoHoursLater = new Date(
                                                            now.getTime() + 2 * 60 * 60 * 1000
                                                        )
                                                        const fifteenDaysLater = new Date(
                                                            now.getTime() + 15 * 24 * 60 * 60 * 1000
                                                        )

                                                        return (
                                                            date < twoHoursLater ||
                                                            date > fifteenDaysLater
                                                        )
                                                    }
                                                "
                                            />
                                        </el-form-item>

                                        <el-form-item label="字幕设置">
                                            <el-checkbox
                                                v-model="currentForm.open_subtitle"
                                                :disabled="templateLoading"
                                            >
                                                开启字幕功能
                                            </el-checkbox>
                                        </el-form-item>

                                        <el-form-item label="互动功能">
                                            <div div class="checkbox-group">
                                                <el-checkbox
                                                    v-model="currentForm.interactive"
                                                    :true-value="1"
                                                    :false-value="0"
                                                    :disabled="templateLoading"
                                                >
                                                    开启
                                                </el-checkbox>
                                            </div>
                                        </el-form-item>

                                        <el-form-item label="加入合集">
                                            <SeasonView
                                                v-model="currentForm.season_id"
                                                v-model:section-id="currentForm.section_id"
                                                :user-uid="selectedUser?.uid"
                                                :disabled="templateLoading"
                                            />
                                        </el-form-item>

                                        <el-form-item label="音质设置">
                                            <div class="checkbox-group">
                                                <el-checkbox
                                                    v-model="currentForm.dolby"
                                                    :true-value="1"
                                                    :false-value="0"
                                                    :disabled="templateLoading"
                                                >
                                                    杜比音效
                                                </el-checkbox>
                                                <el-checkbox
                                                    v-model="currentForm.lossless_music"
                                                    :true-value="1"
                                                    :false-value="0"
                                                    :disabled="templateLoading"
                                                >
                                                    无损音乐
                                                </el-checkbox>
                                            </div>
                                        </el-form-item>

                                        <el-form-item label="内容设置">
                                            <div class="checkbox-group">
                                                <el-checkbox
                                                    v-model="currentForm.no_reprint"
                                                    :true-value="1"
                                                    :false-value="0"
                                                    :disabled="templateLoading"
                                                >
                                                    禁止转载
                                                </el-checkbox>
                                                <el-checkbox
                                                    v-model="currentForm.open_elec"
                                                    :true-value="1"
                                                    :false-value="0"
                                                    :disabled="templateLoading"
                                                >
                                                    开启充电
                                                </el-checkbox>
                                            </div>
                                        </el-form-item>

                                        <el-form-item label="互动管理">
                                            <div class="checkbox-group">
                                                <el-checkbox
                                                    v-model="currentForm.up_selection_reply"
                                                    :true-value="1"
                                                    :false-value="0"
                                                    :disabled="templateLoading"
                                                >
                                                    UP主精选评论
                                                </el-checkbox>
                                                <el-checkbox
                                                    v-model="currentForm.up_close_reply"
                                                    :true-value="1"
                                                    :false-value="0"
                                                    :disabled="templateLoading"
                                                >
                                                    关闭评论
                                                </el-checkbox>
                                                <el-checkbox
                                                    v-model="currentForm.up_close_danmu"
                                                    :true-value="1"
                                                    :false-value="0"
                                                    :disabled="templateLoading"
                                                >
                                                    关闭弹幕
                                                </el-checkbox>
                                            </div>
                                        </el-form-item>

                                        <el-form-item label="可见性">
                                            <el-checkbox
                                                v-model="currentForm.is_only_self"
                                                :true-value="1"
                                                :false-value="0"
                                                :disabled="templateLoading"
                                            >
                                                仅自己可见
                                            </el-checkbox>
                                        </el-form-item>
                                    </div>
                                </el-collapse-transition>
                            </el-card>

                            <!-- 上传操作区域 -->
                            <div class="upload-actions">
                                <el-button
                                    type="primary"
                                    size="large"
                                    :loading="submitting"
                                    @click="submitTemplate"
                                    :disabled="
                                        templateLoading ||
                                        !currentForm.videos ||
                                        currentForm.videos.length === 0 ||
                                        !currentForm.title ||
                                        currentForm.title.trim() === ''
                                    "
                                >
                                    <el-icon v-if="!allFilesUploaded && !submitting"
                                        ><loading
                                    /></el-icon>
                                    <el-icon v-else-if="!submitting"><check /></el-icon>
                                    {{
                                        !getCurrentAutoSubmitting
                                            ? currentTemplate?.aid
                                                ? '编辑稿件'
                                                : '新增稿件'
                                            : '上传完成后自动提交'
                                    }}
                                </el-button>
                                <div class="form-tip" v-if="lastSubmit">
                                    最后提交时间: {{ lastSubmit }}
                                </div>
                            </div>
                        </el-form>
                    </div>
                </div>
            </el-main>
        </el-container>

        <!-- 新建模板 -->
        <NewTemplete
            ref="newTemplateRef"
            v-model="showNewTemplateDialog"
            @template-created="handleTemplateCreated"
        />

        <!-- 登录对话框 -->
        <el-dialog
            v-model="showLoginDialog"
            width="500px"
            :show-close="false"
            :close-on-click-modal="true"
            :close-on-press-escape="false"
            :before-close="handleLoginDialogClose"
            class="login-dialog"
            top="5vh"
        >
            <div class="login-dialog-content" @click.stop>
                <LoginView
                    @login-success="handleLoginSuccess"
                    @loading-change="loginLoading = $event"
                />
            </div>
        </el-dialog>

        <!-- 用户配置弹窗 -->
        <UserConfig v-model="userConfigVisible" :user="configUser" />

        <!-- 视频状对话框 -->
        <VideoStatus
            v-model="showVideoStatusDialog"
            :videos="currentForm?.videos || []"
            :user="selectedUser"
            :template-aid="currentTemplate?.aid"
            @reload-template="
                () =>
                    selectedUser &&
                    currentTemplate?.aid &&
                    reloadTemplateFromAV(selectedUser.uid, currentTemplate.aid)
            "
        />

        <!-- 翻译配置 -->
        <el-dialog
            v-model="showTranslationConfigDialog"
            title="翻译配置"
            width="640px"
            :close-on-click-modal="false"
        >
            <el-form :model="translationConfigForm" label-width="110px">
                <el-form-item label="翻译 API">
                    <el-input
                        v-model="translationConfigForm.apiUrl"
                        placeholder="例如: https://api.openai.com/v1/chat/completions"
                    />
                </el-form-item>
                <el-form-item label="API Key">
                    <el-input
                        v-model="translationConfigForm.apiKey"
                        type="password"
                        show-password
                        placeholder="可留空，不配置则不发送 Authorization"
                    />
                </el-form-item>
                <el-form-item label="模型名称">
                    <el-input
                        v-model="translationConfigForm.model"
                        placeholder="例如: gpt-4o-mini"
                    />
                </el-form-item>
                <el-form-item label="提示词">
                    <el-input
                        v-model="translationConfigForm.prompt"
                        type="textarea"
                        :rows="5"
                        placeholder="请输入翻译提示词"
                    />
                </el-form-item>
                <el-form-item label="自动翻译">
                    <el-switch
                        v-model="translationConfigForm.autoTranslate"
                        active-text="开启"
                        inactive-text="关闭"
                    />
                </el-form-item>
            </el-form>
            <template #footer>
                <div class="dialog-footer">
                    <el-button @click="showTranslationConfigDialog = false">取消</el-button>
                    <el-button
                        type="primary"
                        :loading="savingTranslationConfig"
                        @click="saveTranslationConfig"
                    >
                        保存
                    </el-button>
                </div>
            </template>
        </el-dialog>
        <GlobalConfigView v-model="showGlobalConfigDialog" />
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, nextTick, watch, onUnmounted } from 'vue'
import { v4 as uuidv4 } from 'uuid'
import { useAuthStore } from '../stores/auth'
import { useUserConfigStore, TemplateConfig } from '../stores/user_config'
import { useUtilsStore } from '../stores/utils'
import { useUploadStore } from '../stores/upload'
import { ElMessageBox } from 'element-plus'
import {
    ArrowDown,
    Plus,
    MoreFilled,
    UploadFilled,
    User,
    Check,
    Edit,
    Setting,
    Refresh,
    Delete
} from '@element-plus/icons-vue'
import { open, save } from '@tauri-apps/plugin-dialog'
import { copyFile, remove } from '@tauri-apps/plugin-fs'
import { openUrl } from '@tauri-apps/plugin-opener'
import { listen } from '@tauri-apps/api/event'
import LoginView from '../components/LoginView.vue'
import UserConfig from '../components/UserConfig.vue'
import TopicView from '../components/TopicView.vue'
import SeasonView from '../components/SeasonView.vue'
import UploadQueue from '../components/UploadQueue.vue'
import GlobalConfigView from '../components/GlobalConfig.vue'
import NewTemplete from '../components/NewTemplete.vue'
import VideoList from '../components/VideoList.vue'
import UserList from '../components/UserList.vue'
import VideoStatus from '../components/VideoStatus.vue'
import TagView from '../components/TagView.vue'
import PublishTimeEditor from '../components/PublishTimeEditor.vue'

const authStore = useAuthStore()
const userConfigStore = useUserConfigStore()
const uploadStore = useUploadStore()
const utilsStore = useUtilsStore()

// 计算属?
const loginUsers = computed(() => authStore.loginUsers)
const userTemplates = computed(() => userConfigStore.userTemplates)
const typeList = computed(() => utilsStore.typelist)

const currentVer = ref<string>('')

// ʾURL
const coverDisplayUrl = ref<string>('')
const coverLoading = ref<boolean>(false)

// 响应式数?
const selectedUser = ref<any>(null)
const currentTemplateName = ref<string>('')
type MainPanelMode = 'upload' | 'publishTimeEdit'
const mainPanelMode = ref<MainPanelMode>('upload')
const isUploadMode = computed(() => mainPanelMode.value === 'upload')
const isPublishTimeEditMode = computed(() => mainPanelMode.value === 'publishTimeEdit')
const switchToUploadMode = () => {
    mainPanelMode.value = 'upload'
}
const switchToPublishTimeEditMode = () => {
    mainPanelMode.value = 'publishTimeEdit'
}
const showNewTemplateDialog = ref(false)
const showLoginDialog = ref(false)
const showGlobalConfigDialog = ref(false)
const showTranslationConfigDialog = ref(false)
const loginLoading = ref(false)
const uploading = ref(false)
const submitting = ref(false)
const templateLoading = ref(false) // 模板加载状锁

// 视频状对话框
const showVideoStatusDialog = ref(false)

const DEFAULT_TRANSLATION_PROMPT =
    'You are a professional video title translator. Translate the input title into concise, natural Simplified Chinese. Keep product names, proper nouns, and abbreviations accurate. Output only the translated title without explanation or quotes.'

const translationConfigForm = ref({
    apiUrl: '',
    apiKey: '',
    model: '',
    prompt: DEFAULT_TRANSLATION_PROMPT,
    autoTranslate: false
})
const savingTranslationConfig = ref(false)
const titleTranslating = ref(false)
let translationRequestToken = 0

const canTranslateTitle = computed(() => {
    return Boolean(currentForm.value?.title?.trim())
})

// 组件引用
const newTemplateRef = ref<InstanceType<typeof NewTemplete> | null>(null)
const tagViewRef = ref<InstanceType<typeof TagView> | null>(null)
// 自动提交状记?- 记录每个模板的自动提交状?
const autoSubmittingRecord = ref<Record<string, boolean>>({})
const autoSubmittingGroupRecord = ref<Record<string, string[]>>({})
// 全局自动提交棢查间?
let autoSubmitInterval: number | null = null

// 高亮显示自动提交状的弢?
const highlightAutoSubmitting = ref<boolean>(
    localStorage.getItem('highlightAutoSubmitting') === 'true'
)

// |ر仯浽localStorage
watch(highlightAutoSubmitting, newValue => {
    localStorage.setItem('highlightAutoSubmitting', String(newValue))
})

// 生成模板键名
const getTemplateKey = (uid: number, templateName: string) => `${uid}-${templateName}`

// 获取当前模板的自动提交状?
const getCurrentAutoSubmitting = computed(() => {
    if (!selectedUser.value || !currentTemplateName.value) return false
    const key = getTemplateKey(selectedUser.value.uid, currentTemplateName.value)
    return autoSubmittingRecord.value[key] || false
})

// 设置模板的自动提交状?
const setAutoSubmitting = (uid: number, templateName: string, status: boolean) => {
    const key = getTemplateKey(uid, templateName)
    if (status) {
        autoSubmittingRecord.value[key] = true
    } else {
        delete autoSubmittingRecord.value[key]
        delete autoSubmittingGroupRecord.value[key]
    }
}

// 棢查是否有任何模板在自动提?
const hasAnyAutoSubmitting = computed(() => {
    return Object.keys(autoSubmittingRecord.value).length > 0
})

// ȫԶύ麯?
const checkAutoSubmitAll = async () => {
    const templateKeys = Object.keys(autoSubmittingRecord.value)

    for (const templateKey of templateKeys) {
        const [uidStr, templateName] = templateKey.split('-', 2)
        const uid = parseInt(uidStr)

        if (isNaN(uid) || !templateName) continue

        // 获取用户和模板配?
        const user = loginUsers.value.find(u => u.uid === uid)
        if (!user || !userConfigStore.configRoot?.config[uid]?.templates[templateName]) {
            // ûģ岻ڣԶύ״?
            setAutoSubmitting(uid, templateName, false)
            continue
        }

        const template = userConfigStore.configRoot.config[uid].templates[templateName]

        // 棢查是否所有文件都已上传完?
        if (template.videos && template.videos.length > 0) {
            const targetVideoIds = autoSubmittingGroupRecord.value[templateKey] || []
            const targetVideos =
                targetVideoIds.length > 0
                    ? template.videos.filter(video => targetVideoIds.includes(String(video.id || '')))
                    : template.videos

            const allUploaded =
                targetVideos.length > 0 &&
                targetVideos.every(video => video.complete && video.path === '')

            if (allUploaded && autoSubmittingRecord.value[templateKey]) {
                // 目标文件已全部上传完成，执行提交
                setAutoSubmitting(uid, templateName, false)
                try {
                    const submitTemplateData = {
                        ...template,
                        videos: targetVideos
                    }
                    await performTemplateSubmit(uid, templateName, submitTemplateData)
                } catch (error) {
                    console.error(`ģ ${templateKey} 自动提交失败:`, error)
                }
            }
        } else {
            // 没有视频文件，清除自动提交状?
            setAutoSubmitting(uid, templateName, false)
        }
    }

    // 如果没有模板在自动提交，停止间隔棢?
    if (!hasAnyAutoSubmitting.value && autoSubmitInterval) {
        clearInterval(autoSubmitInterval)
        autoSubmitInterval = null
    }
}

// 启动全局自动提交棢?
const startAutoSubmitCheck = () => {
    if (!autoSubmitInterval) {
        autoSubmitInterval = setInterval(checkAutoSubmitAll, 1000)
    }
}

// 执行模板提交
const performTemplateSubmit = async (uid: number, templateName: string, template: any) => {
    const user = loginUsers.value.find(u => u.uid === uid)
    if (!user) throw new Error('用户不存在')

    submitting.value = true
    try {
        const resp = (await uploadStore.submitTemplate(uid, template)) as any

        // 更新朢后提交时间（只对当前模板?
        if (selectedUser.value?.uid === uid && currentTemplateName.value === templateName) {
            lastSubmit.value = new Date().toLocaleString()
        }

        utilsStore.showMessage(`视频 ${resp.bvid} 提交成功（模板: ${templateName}）`, 'success')
        console.log(`视频 ${resp.bvid} 提交成功（模板: ${templateName}）`)

        if (!template?.aid) {
            const targetTemplate =
                userConfigStore.configRoot?.config[uid]?.templates?.[templateName] || null
            if (targetTemplate?.videos && Array.isArray(targetTemplate.videos)) {
                const groupedByKey = new Map<string, Set<string>>()
                for (const video of template.videos || []) {
                    const { grouped: isGrouped, key, role } = getDerivedVideoGroup(video)
                    if (!isGrouped || !role) continue
                    if (!groupedByKey.has(key)) groupedByKey.set(key, new Set<string>())
                    groupedByKey.get(key)!.add(role)
                }

                const submittedGroupKeys = new Set(
                    Array.from(groupedByKey.entries())
                        .filter(([, roles]) => roles.has('中配') && roles.has('熟肉'))
                        .map(([key]) => key)
                )

                if (submittedGroupKeys.size > 0) {
                    targetTemplate.videos = targetTemplate.videos.filter(video => {
                        const { grouped: isGrouped, key } = getDerivedVideoGroup(video)
                        if (!isGrouped) return true
                        return !submittedGroupKeys.has(key)
                    })

                    if (selectedUser.value?.uid === uid && currentTemplateName.value === templateName) {
                        syncTitleFromGroupedVideos()
                        await autoTranslateCurrentTitle()
                    }
                }
            }
        }

        if (resp && resp.aid && utilsStore.hasSeason) {
            try {
                const old_season_id = await utilsStore.getVideoSeason(uid, resp.aid)
                let add = old_season_id && old_season_id !== 0 ? false : true

                if (template && old_season_id !== template.season_id && template.videos[0]?.cid) {
                    const new_season_id = template.season_id || 0
                    const new_section_id = template.section_id || 0
                    await utilsStore.switchSeason(
                        uid,
                        resp.aid,
                        template.videos[0]?.cid,
                        new_season_id,
                        new_section_id,
                        template.title,
                        add
                    )

                    const season_title =
                        utilsStore.seasonlist.find((s: any) => s.season_id === template.season_id)
                            ?.title || template.season_id
                    utilsStore.showMessage(`视频 ${resp.bvid} 已加入合集 ${season_title}`, 'success')
                    console.log(`视频 ${resp.bvid} 已加入合集 ${season_title}`)
                }
            } catch (error) {
                console.error('加入合集失败: ', error)
                utilsStore.showMessage(`加入合集失败: ${error}`, 'error')
            }
        }

        setTimeout(async () => {
            try {
                if (!template.aid) {
                    const userConfig = userConfigStore.configRoot?.config[uid]
                    if (userConfig && userConfig.auto_edit && newTemplateRef.value) {
                        // 新增稿件且auto_edit弢启，创建编辑模板
                        await newTemplateRef.value.createTemplateFromBV(
                            uid,
                            resp.bvid,
                            resp.bvid,
                            true
                        )
                        utilsStore.showMessage('从 BV 号创建模板成功', 'success')
                    }
                } else {
                    if (
                        selectedUser.value?.uid === uid &&
                        currentTemplateName.value === templateName
                    ) {
                        reloadTemplateFromAV(uid, template.aid)
                    }
                }
            } catch (error) {
                utilsStore.showMessage(`${error}`, 'error')
            } finally {
                submitting.value = false
            }
        }, 500)
    } catch (error) {
        console.error('模板提交失败:', error)
        utilsStore.showMessage(`模板提交失败: ${error}`, 'error')
        submitting.value = false
    }
}
const lastSubmit = ref<string>('')

// 卡片折叠状?
const cardCollapsed = ref({
    basic: false, // Ϣ
    tags: false, // 标签设置
    description: false, // 视频描述
    videos: false, // 视频文件
    advanced: false // ߼ѡ
})

// 模板名编辑相?
const isEditingTemplateName = ref(false)
const editingTemplateName = ref('')
const templateNameInputRef = ref()

// 拖拽状?
const isDragOver = ref(false)

// 内容容器引用
const contentWrapperRef = ref<HTMLElement | null>(null)

// û
const userConfigVisible = ref(false)
const configUser = ref<any>(null)

// 分区数据
const selectedCategory = ref<any>(null)
const selectedSubCategory = ref<any>(null)
const categoryPopoverVisible = ref(false)
let generalUpdateTimer: number | null = null

const currentTemplate = computed(() => {
    if (!selectedUser.value || !currentTemplateName.value || !userConfigStore.configRoot?.config) {
        return null
    }
    const userConfig = userConfigStore.configRoot.config[selectedUser.value.uid]
    if (!userConfig || !userConfig.templates[currentTemplateName.value]) {
        return null
    }
    return userConfig.templates[currentTemplateName.value]
})

// ǰ - ֱӲģ
const currentForm = computed({
    get() {
        return currentTemplate.value
    },
    set(value) {
        if (
            !selectedUser.value ||
            !currentTemplateName.value ||
            !userConfigStore.configRoot?.config ||
            !value
        ) {
            return
        }
        const userConfig = userConfigStore.configRoot.config[selectedUser.value.uid]
        if (userConfig && userConfig.templates[currentTemplateName.value]) {
            userConfig.templates[currentTemplateName.value] = value
        }
    }
})

const tags = ref<string[]>([])

// 日期选择器的计算属?- 处理时间戳转?
const dtimeDate = computed({
    get() {
        return currentForm.value?.dtime ? new Date(currentForm.value.dtime * 1000) : null
    },
    set(value: Date | null) {
        if (currentForm.value) {
            currentForm.value.dtime = value ? Math.floor(value.getTime() / 1000) : undefined
        }
    }
})

// 视频数组的计算属?- 确保始终返回数组
const videos = computed({
    get() {
        return currentForm.value?.videos || []
    },
    set(value: any[]) {
        if (currentForm.value) {
            currentForm.value.videos = value
        }
    }
})

// 棢查指定模板是否有未保存的改动
const checkTemplateHasUnsavedChanges = (uid: number, templateName: string): boolean => {
    if (!userConfigStore.configRoot?.config || !userConfigStore.configBase?.config) {
        return false
    }

    const currentUserConfig = userConfigStore.configRoot.config[uid]
    const baseUserConfig = userConfigStore.configBase.config[uid]

    if (!currentUserConfig?.templates[templateName] || !baseUserConfig?.templates[templateName]) {
        return false
    }

    const currentTemplateData = currentUserConfig.templates[templateName]
    const baseTemplateData = baseUserConfig.templates[templateName]

    return hasUnsavedChanges(baseTemplateData, currentTemplateData)
}

// 生命周期
// 仯첽ʾõķURL
watch(
    () => currentForm.value?.cover,
    async (newCover: string | undefined) => {
        if (newCover && selectedUser.value) {
            try {
                coverLoading.value = true
                const downloadedCover = await utilsStore.downloadCover(
                    selectedUser.value.uid,
                    newCover
                )
                coverDisplayUrl.value = downloadedCover || ''
            } catch (error) {
                console.error('Failed to download cover:', error)
                coverDisplayUrl.value = ''
            } finally {
                coverLoading.value = false
            }
        } else {
            coverDisplayUrl.value = ''
            coverLoading.value = false
        }
    }
)

// ǩ仯±?
watch(
    () => tags.value,
    (newTags: string[]) => {
        if (currentForm.value) {
            currentForm.value.tag = newTags.join(',')
        }
    },
    { deep: true }
)

// ǩ仯±ǩ?
watch(
    () => currentForm.value?.tag,
    (newTag: string | undefined) => {
        const newTags = newTag ? newTag.split(',').filter(tag => tag.trim()) : []
        if (JSON.stringify(newTags) !== JSON.stringify(tags.value)) {
            tags.value = newTags
        }
    }
)

// 仯·˫󶨣
watch(
    () => currentForm.value?.tid,
    (newTid: number | undefined) => {
        if (newTid && newTid > 0) {
            // 根据tid设置选中的分?
            setSelectedCategoryByTid(newTid)
        } else {
            // 如果没有分区信息，清空分区择
            selectedCategory.value = null
            selectedSubCategory.value = null
        }
    }
)

// 监听用户切换，重新加载封?
watch(
    () => selectedUser.value,
    async (newUser: any) => {
        if (currentForm.value?.cover && newUser) {
            try {
                coverLoading.value = true
                const downloadedCover = await utilsStore.downloadCover(
                    newUser.uid,
                    currentForm.value.cover
                )
                coverDisplayUrl.value = downloadedCover || ''
            } catch (error) {
                console.error('Failed to download cover:', error)
                coverDisplayUrl.value = ''
            } finally {
                coverLoading.value = false
            }
        } else {
            coverDisplayUrl.value = ''
            coverLoading.value = false
        }
    }
)

let keyboardCleanup: (() => void) | null = null
let dragAndDropCleanup: (() => void) | null = null

const forwardConsole = (fnName: keyof Console, logger: (level: string, ...args: any[]) => void) => {
    const original = console[fnName] as (...args: any[]) => void
    ;(console as any)[fnName] = (...args: any[]) => {
        original(...args)
        logger(fnName as string, ...args)
    }
}

onMounted(async () => {
    await initializeData()
    await setupDragAndDrop()
    dragAndDropCleanup = setupDomFileDragDrop()
    keyboardCleanup = await setupKeyboardShortcuts()

    forwardConsole('log', utilsStore.log)
    forwardConsole('error', utilsStore.log)
    forwardConsole('warn', utilsStore.log)

    // Ҽ˵ˢ
    document.addEventListener('contextmenu', (event: MouseEvent) => {
        event.preventDefault()
    })
})

// жʱ
onUnmounted(() => {
    if (keyboardCleanup) {
        keyboardCleanup()
    }
    if (dragAndDropCleanup) {
        dragAndDropCleanup()
        dragAndDropCleanup = null
    }

    // 清理自动提交间隔棢?
    if (autoSubmitInterval) {
        clearInterval(autoSubmitInterval)
        autoSubmitInterval = null
    }

    if (generalUpdateTimer) {
        clearInterval(generalUpdateTimer)
        generalUpdateTimer = null
    }

    // 清理扢有自动提交状?    autoSubmittingRecord.value = {}
    autoSubmittingGroupRecord.value = {}
})

// 初始化数?
const initializeData = async () => {
    try {
        currentVer.value = (await utilsStore.getCurrentVersion()) as string
        // ȡ¼û
        await authStore.getLoginUsers()

        // ûģб
        if (loginUsers.value.length > 0) {
            await utilsStore.initTypeList(loginUsers.value[0].uid)
            await utilsStore.initTopicList(loginUsers.value[0].uid)
            await userConfigStore.buildUserTemplates(loginUsers.value)
            await uploadStore.getUploadQueue()
            if (!generalUpdateTimer) {
                generalUpdateTimer = setInterval(() => {
                    if (authStore.loginUsers.length > 0) {
                        uploadStore.getUploadQueue()
                    }
                    for (const task of uploadStore.uploadQueue) {
                        if (task.status === 'Completed') {
                            const templateName = task.template
                            const uid = task.user.uid
                            const videos =
                                userConfigStore.configRoot?.config[uid]?.templates[templateName]
                                    ?.videos || []

                            const video = videos.find(v => v.id === task.video?.id)
                            if (video && video.filename !== task.video?.filename) {
                                video.filename = task.video.filename
                                video.path = task.video.path
                                video.complete = true
                                video.finished_at = task.finished_at
                            }
                        }
                    }
                }, 666) // ϴ
            }
        }

        setTimeout(async () => {
            await restoreTemplateSelection()
            restoreCardCollapsedState()
        }, 100)
    } catch (error) {
        console.error('初始化数据失败: ', error)
        utilsStore.showMessage(`初始化数据失败: ${error}`, 'error')
    }
}

const hasUnsavedChanges = (
    baseTemplateData: TemplateConfig,
    currentTemplateData: TemplateConfig
) => {
    // 比较关键字段
    const fieldsToCompare = [
        'title',
        'cover',
        'copyright',
        'source',
        'tid',
        'desc',
        'dynamic',
        'tag',
        'dtime',
        'open_subtitle',
        'interactive',
        'mission_id',
        'topic_id',
        'season_id',
        'section_id',
        'dolby',
        'lossless_music',
        'no_reprint',
        'open_elec',
        'up_selection_reply',
        'up_close_reply',
        'up_close_danmu',
        'is_only_self',
        'watermark'
    ]

    for (const field of fieldsToCompare) {
        const currentValue = (currentTemplateData as any)[field]
        const baseValue = (baseTemplateData as any)[field]

        // 处理 undefined/null/ַ 的情?
        if (
            (currentValue === undefined || currentValue === null || currentValue === '') &&
            (baseValue === undefined || baseValue === null || baseValue === '')
        ) {
            continue
        }

        if (JSON.stringify(currentValue) !== JSON.stringify(baseValue)) {
            // console.log(field, '有改变')
            // console.log('current: ', JSON.stringify(currentValue), 'vs', JSON.stringify(baseValue))
            return true
        }
    }

    // رȽ videos 数组
    const currentVideos = currentTemplateData.videos || []
    const baseVideos = baseTemplateData.videos || []

    if (currentVideos.length !== baseVideos.length) {
        return true
    }

    // 比较视频的关键字?
    for (let i = 0; i < currentVideos.length; i++) {
        const currentVideo = currentVideos[i]
        const baseVideo = baseVideos[i]

        const videoFieldsToCompare = ['title', 'filename', 'desc', 'path', 'cid']
        for (const field of videoFieldsToCompare) {
            if (
                JSON.stringify((currentVideo as any)[field]) !==
                JSON.stringify((baseVideo as any)[field])
            ) {
                return true
            }
        }
    }

    return false
}

// ק
const setupDragAndDrop = async () => {
    try {
        // ļק¼
        await listen('tauri://drag-drop', async event => {
            const videos = event.payload as string[]
            isDragOver.value = false
            if (templateLoading.value) {
                utilsStore.showMessage('模板加载中，请稍后再试', 'warning')
                return
            }
            await handleDroppedFiles(videos)
        })

        // 监听拖拽悬停事件
        await listen('tauri://drag-over', event => {
            if (!isDragOver.value) console.log('文件拖拽悬停:', event.payload, '，忽略后续日志')
            isDragOver.value = true
        })

        // קȡ¼
        await listen('tauri://drag-leave', () => {
            console.log('文件拖拽离开')
            isDragOver.value = false
        })
    } catch (error) {
        console.error('设置拖拽监听失败: ', error)
        utilsStore.showMessage(`设置拖拽监听失败: ${error}`, 'error')
    }
}

const isDomFileDragEvent = (event: DragEvent) => {
    const dataTransfer = event.dataTransfer
    if (!dataTransfer) return false
    return Array.from(dataTransfer.types || []).includes('Files')
}

const buildDomDroppedFilePayload = (event: DragEvent) => {
    const files = Array.from(event.dataTransfer?.files || [])
    const paths = files
        .map(file => {
            const anyFile = file as File & { path?: string }
            return anyFile.path || file.webkitRelativePath || ''
        })
        .filter(path => !!path)
    return {
        files,
        paths,
        value: paths.join('\n')
    }
}

const setupDomFileDragDrop = () => {
    let dragDepth = 0

    const handleDragEnter = (event: DragEvent) => {
        if (!isDomFileDragEvent(event)) return
        if (!isUploadMode.value) return
        event.preventDefault()
        dragDepth += 1
        isDragOver.value = true
    }

    const handleDragOver = (event: DragEvent) => {
        if (!isDomFileDragEvent(event)) return
        if (!isUploadMode.value) return
        event.preventDefault()
        if (event.dataTransfer) {
            event.dataTransfer.dropEffect = 'copy'
        }
        isDragOver.value = true
    }

    const handleDragLeave = (event: DragEvent) => {
        if (!isDomFileDragEvent(event)) return
        if (!isUploadMode.value) return
        event.preventDefault()
        dragDepth = Math.max(0, dragDepth - 1)
        if (dragDepth === 0) {
            isDragOver.value = false
        }
    }

    const handleDrop = async (event: DragEvent) => {
        if (!isDomFileDragEvent(event)) return
        if (!isUploadMode.value) return
        event.preventDefault()
        dragDepth = 0
        isDragOver.value = false
        await handleDroppedFiles(buildDomDroppedFilePayload(event))
    }

    document.addEventListener('dragenter', handleDragEnter)
    document.addEventListener('dragover', handleDragOver)
    document.addEventListener('dragleave', handleDragLeave)
    document.addEventListener('drop', handleDrop)

    return () => {
        document.removeEventListener('dragenter', handleDragEnter)
        document.removeEventListener('dragover', handleDragOver)
        document.removeEventListener('dragleave', handleDragLeave)
        document.removeEventListener('drop', handleDrop)
        dragDepth = 0
        isDragOver.value = false
    }
}

// 设置键盘快捷?
const setupKeyboardShortcuts = async () => {
    const handleKeydown = (event: KeyboardEvent) => {
        // 禁用 F5 ˢ
        if (!event.ctrlKey && event.key === 'F5') {
            event.preventDefault()
            return
        }

        // Ctrl+F5 ˢҳ
        if (event.ctrlKey && event.key === 'F5') {
            event.preventDefault()
            window.location.reload()
            return
        }

        if (event.ctrlKey && event.key === 'r') {
            event.preventDefault()
            if (selectedUser.value && currentTemplateName.value) {
                resetTemplate()
            }
            return
        }

        // Ctrl+S ģ
        if (event.ctrlKey && event.key === 's') {
            event.preventDefault()
            if (selectedUser.value && currentTemplateName.value) {
                saveTemplate()
            }
        }
    }

    document.addEventListener('keydown', handleKeydown)

    // 返回清理函数
    return () => {
        document.removeEventListener('keydown', handleKeydown)
    }
}

// 切换卡片折叠状?
const toggleCardCollapsed = (cardKey: keyof typeof cardCollapsed.value) => {
    cardCollapsed.value[cardKey] = !cardCollapsed.value[cardKey]
    // 保存折叠状到localStorage
    saveCardCollapsedState()
}

// ģѡ书
const TEMPLATE_SELECTION_KEY = 'last-selected-template'
const CARD_COLLAPSED_KEY = 'card-collapsed-state'

// Ƿڻָģѡ񣨱鱣?
const isRestoringTemplate = ref(false)

// ģѡlocalStorage
const saveTemplateSelection = (userUid: number, templateName: string) => {
    // ڻָģ壬棨?
    if (isRestoringTemplate.value) return

    try {
        const selection = {
            userUid,
            templateName,
            timestamp: Date.now()
        }
        localStorage.setItem(TEMPLATE_SELECTION_KEY, JSON.stringify(selection))
    } catch (error) {
        console.error('保存模板选择失败:', error)
    }
}

// 濨Ƭ۵״?
const saveCardCollapsedState = () => {
    try {
        localStorage.setItem(CARD_COLLAPSED_KEY, JSON.stringify(cardCollapsed.value))
    } catch (error) {
        console.error('保存卡片折叠状态失败:', error)
    }
}

// 恢复卡片折叠状?
const restoreCardCollapsedState = () => {
    try {
        const saved = localStorage.getItem(CARD_COLLAPSED_KEY)
        if (saved) {
            const savedState = JSON.parse(saved)
            Object.assign(cardCollapsed.value, savedState)
        }
    } catch (error) {
        console.error('恢复卡片折叠状态失败:', error)
    }
}

// 从localStorage恢复模板选择
const restoreTemplateSelection = async () => {
    try {
        const saved = localStorage.getItem(TEMPLATE_SELECTION_KEY)
        if (!saved) return

        const selection = JSON.parse(saved)
        const { userUid, templateName, timestamp } = selection

        // 棢查数据有效（超过30天的记录自动失效?
        const thirtyDaysAgo = Date.now() - 30 * 24 * 60 * 60 * 1000
        if (timestamp && timestamp < thirtyDaysAgo) {
            localStorage.removeItem(TEMPLATE_SELECTION_KEY)
            return
        }

        // 棢查用户是否仍然登?
        const targetUser = loginUsers.value.find(user => user.uid === userUid)
        if (!targetUser) {
            // 用户已不存在，清除记?
            localStorage.removeItem(TEMPLATE_SELECTION_KEY)
            return
        }

        // 棢查模板是否仍然存?
        const userTemplate = userTemplates.value.find(ut => ut.user.uid === userUid)
        const template = userTemplate?.templates.find(t => t.name === templateName)
        if (!template) {
            // 模板已不存在，清除记?
            localStorage.removeItem(TEMPLATE_SELECTION_KEY)
            return
        }

        // 确保用户是展弢状?
        if (userTemplate && !userTemplate.expanded) {
            toggleUserExpanded(userUid)
        }

        // 设置恢复状标?
        isRestoringTemplate.value = true

        // 自动选择模板
        await selectTemplate(targetUser, templateName)

        // 恢复完成后重置标?
        isRestoringTemplate.value = false

        console.log(`自动恢复模板选择: ${targetUser.username} - ${templateName}`)
        utilsStore.showMessage(`已恢复上次选择的模板: ${templateName}`, 'success')
    } catch (error) {
        console.error('恢复模板选择失败:', error)
        // ָʧܣЧĴ洢
        localStorage.removeItem(TEMPLATE_SELECTION_KEY)
    } finally {
        // 确保标志被重?
        isRestoringTemplate.value = false
    }
}

// 清空卡片内容
const clearCardContent = async (cardType: 'basic' | 'tags' | 'description' | 'advanced') => {
    if (!currentForm.value) {
        utilsStore.showMessage('请先选择模板', 'warning')
        return
    }

    // ڼģ壬ֹ?
    if (templateLoading.value) {
        utilsStore.showMessage('模板正在加载中，请稍后再试', 'warning')
        return
    }

    try {
        // 确认清空
        await ElMessageBox.confirm(
            `确定要清空${getCardDisplayName(cardType)}的所有内容吗？`,
            '确认清空',
            {
                confirmButtonText: '确定',
                cancelButtonText: '取消',
                type: 'warning'
            }
        )

        // 根据卡片类型清空相应内容
        switch (cardType) {
            case 'basic':
                currentForm.value.title_prefix = ''
                currentForm.value.title = ''
                currentForm.value.cover = ''
                currentForm.value.tid = 0
                currentForm.value.copyright = 1
                currentForm.value.source = ''
                // 同步清空分区选择状?
                selectedCategory.value = null
                selectedSubCategory.value = null
                // շʾ
                coverDisplayUrl.value = ''
                break

            case 'tags':
                currentForm.value.tag = ''
                // 同步清空标签数组
                tags.value = []
                // ͨTagView的状?
                tagViewRef.value?.clearTags()
                break

            case 'description':
                currentForm.value.desc = ''
                currentForm.value.dynamic = ''
                break

            case 'advanced':
                currentForm.value.watermark =
                    userConfigStore?.configRoot?.config[selectedUser.value.uid]?.watermark || 0
                currentForm.value.dtime = undefined
                currentForm.value.interactive = 0
                currentForm.value.dolby = 0
                currentForm.value.lossless_music = 0
                currentForm.value.no_reprint = 0
                currentForm.value.open_elec = 0
                currentForm.value.up_selection_reply = 0
                currentForm.value.up_close_reply = 0
                currentForm.value.up_close_danmu = 0
                currentForm.value.atomic_int = 0
                currentForm.value.is_only_self = 0
                break
        }

        utilsStore.showMessage(`已清空${getCardDisplayName(cardType)}的内容`, 'success')
    } catch (error) {
        // 用户取消了操?
    }
}

// ȡƬʾ
const getCardDisplayName = (cardType: string): string => {
    const cardNames: Record<string, string> = {
        basic: '基本信息',
        tags: '标签设置',
        description: '视频描述',
        videos: '视频文件',
        advanced: '高级设置'
    }
    return cardNames[cardType] || cardType
}

type DubbingRole = '中配' | '熟肉'

const normalizeVideoPath = (videoPath: string) => videoPath.trim().replace(/\//g, '\\').toLowerCase()

const normalizeGroupKey = (title: string) =>
    title.normalize('NFKC').replace(/\u3000/g, ' ').replace(/\s+/g, ' ').trim()

const parseVideoNameForGrouping = (nameWithoutExt: string) => {
    const displayTitle = nameWithoutExt.trim()
    const patterns = [
        /^[\uFF08(\u3010\[]\s*(\u4E2D\u914D|\u719F\u8089)\s*[\uFF09)\u3011\]]\s*(.+)$/u,
        /^(\u4E2D\u914D|\u719F\u8089)\s*[-_ ]+\s*(.+)$/u
    ]

    let matched: RegExpMatchArray | null = null
    for (const pattern of patterns) {
        matched = displayTitle.match(pattern)
        if (matched) break
    }

    if (!matched) {
        return {
            role: null as DubbingRole | null,
            baseTitle: '',
            displayTitle
        }
    }

    const baseTitle = normalizeGroupKey(matched[2] || '')
    if (!baseTitle) {
        return {
            role: null as DubbingRole | null,
            baseTitle: '',
            displayTitle
        }
    }

    return {
        role: matched[1] as DubbingRole,
        baseTitle,
        displayTitle
    }
}

const getDerivedVideoGroup = (video: any) => {
    const keyFromField = normalizeGroupKey(String(video?.group_key || ''))
    const roleFromField = String(video?.group_role || '').trim()
    if (keyFromField && (roleFromField === '中配' || roleFromField === '熟肉')) {
        return {
            grouped: true,
            key: keyFromField,
            role: roleFromField as DubbingRole
        }
    }

    const candidateTexts = [
        typeof video?.title === 'string' ? video.title : '',
        typeof video?.filename === 'string' ? video.filename : '',
        typeof video?.path === 'string'
            ? (video.path.split(/[/\\]/).pop() as string) || video.path
            : ''
    ]

    for (const candidate of candidateTexts) {
        const text = String(candidate || '').trim()
        if (!text) continue
        const nameWithoutExt = text.replace(/\.[^/.]+$/, '')
        const parsed = parseVideoNameForGrouping(nameWithoutExt)
        if (parsed.role && parsed.baseTitle) {
            return {
                grouped: true,
                key: parsed.baseTitle,
                role: parsed.role
            }
        }
    }

    return {
        grouped: false,
        key: '',
        role: null as DubbingRole | null
    }
}

const hasMatchedVideoGroups = (videos: any[]) => {
    const grouped = new Map<string, Set<string>>()
    for (const video of videos || []) {
        const { grouped: isGrouped, key, role } = getDerivedVideoGroup(video)
        if (!isGrouped || !role) continue
        if (!grouped.has(key)) grouped.set(key, new Set())
        grouped.get(key)!.add(role)
    }
    return Array.from(grouped.values()).some(roles => roles.has('中配') && roles.has('熟肉'))
}

const syncTitleFromGroupedVideos = () => {
    if (!currentForm.value) return
    const grouped = new Map<string, Set<string>>()
    for (const video of currentForm.value.videos || []) {
        const { grouped: isGrouped, key, role } = getDerivedVideoGroup(video)
        if (!isGrouped || !role) continue
        if (!grouped.has(key)) grouped.set(key, new Set())
        grouped.get(key)!.add(role)
    }
    const firstMatched = Array.from(grouped.entries()).find(([, roles]) => {
        return roles.has('中配') && roles.has('熟肉')
    })
    if (firstMatched) {
        currentForm.value.title = firstMatched[0]
    }
}

const getTranslationConfig = () => {
    return {
        apiUrl: String(userConfigStore.configRoot?.translation_api_url || '').trim(),
        apiKey: String(userConfigStore.configRoot?.translation_api_key || '').trim(),
        model: String(userConfigStore.configRoot?.translation_model || '').trim(),
        prompt:
            String(userConfigStore.configRoot?.translation_prompt || '').trim() ||
            DEFAULT_TRANSLATION_PROMPT,
        autoTranslate: Boolean(userConfigStore.configRoot?.translation_auto)
    }
}

const openTranslationConfigDialog = async () => {
    try {
        if (!userConfigStore.configRoot) {
            await userConfigStore.loadConfig()
        }
        const config = getTranslationConfig()
        translationConfigForm.value = {
            apiUrl: config.apiUrl,
            apiKey: config.apiKey,
            model: config.model,
            prompt: config.prompt,
            autoTranslate: config.autoTranslate
        }
        showTranslationConfigDialog.value = true
    } catch (error) {
        console.error('打开翻译配置失败:', error)
        utilsStore.showMessage(`打开翻译配置失败: ${error}`, 'error')
    }
}

const saveTranslationConfig = async () => {
    savingTranslationConfig.value = true
    try {
        if (!userConfigStore.configRoot) {
            await userConfigStore.loadConfig()
        }

        const payload = {
            translation_api_url: translationConfigForm.value.apiUrl.trim(),
            translation_api_key: translationConfigForm.value.apiKey.trim(),
            translation_model: translationConfigForm.value.model.trim(),
            translation_prompt:
                translationConfigForm.value.prompt.trim() || DEFAULT_TRANSLATION_PROMPT,
            translation_auto: translationConfigForm.value.autoTranslate
        }

        await userConfigStore.updateGlobalConfig(payload)
        showTranslationConfigDialog.value = false
        utilsStore.showMessage('翻译配置已保存', 'success')
    } catch (error) {
        console.error('保存翻译配置失败:', error)
        utilsStore.showMessage(`保存翻译配置失败: ${error}`, 'error')
    } finally {
        savingTranslationConfig.value = false
    }
}

const buildTranslationEndpoint = (apiUrl: string): string => {
    const trimmed = apiUrl.trim().replace(/\/+$/, '')
    if (!trimmed) return ''
    if (/\/chat\/completions$/i.test(trimmed)) return trimmed
    if (/\/v1$/i.test(trimmed)) return `${trimmed}/chat/completions`
    return `${trimmed}/v1/chat/completions`
}

const extractTranslatedText = (responseData: any): string => {
    const messageContent = responseData?.choices?.[0]?.message?.content
    if (typeof messageContent === 'string') {
        return messageContent.trim()
    }
    if (Array.isArray(messageContent)) {
        return messageContent
            .map(item => {
                if (typeof item === 'string') return item
                if (item && typeof item.text === 'string') return item.text
                return ''
            })
            .join('')
            .trim()
    }

    const fallbackCandidates = [
        responseData?.output_text,
        responseData?.translation,
        responseData?.result
    ]
    for (const candidate of fallbackCandidates) {
        if (typeof candidate === 'string' && candidate.trim()) {
            return candidate.trim()
        }
    }
    return ''
}

const translateCurrentTitle = async (manual = false) => {
    if (!currentForm.value) return
    const sourceTitle = String(currentForm.value.title || '').trim()
    if (!sourceTitle) {
        if (manual) {
            utilsStore.showMessage('请先填写视频标题', 'warning')
        }
        return
    }

    const config = getTranslationConfig()
    if (!config.apiUrl || !config.model) {
        if (manual) {
            utilsStore.showMessage('请先填写翻译 API 和模型名称', 'warning')
        }
        return
    }

    const endpoint = buildTranslationEndpoint(config.apiUrl)
    if (!endpoint) {
        if (manual) {
            utilsStore.showMessage('翻译 API 地址无效', 'warning')
        }
        return
    }

    const currentToken = ++translationRequestToken
    titleTranslating.value = true

    try {
        const headers: Record<string, string> = {
            'Content-Type': 'application/json'
        }
        if (config.apiKey) {
            headers.Authorization = `Bearer ${config.apiKey}`
        }

        const response = await fetch(endpoint, {
            method: 'POST',
            headers,
            body: JSON.stringify({
                model: config.model,
                messages: [
                    { role: 'system', content: config.prompt },
                    { role: 'user', content: sourceTitle }
                ],
                temperature: 0.2
            })
        })

        if (!response.ok) {
            const errText = await response.text().catch(() => '')
            throw new Error(`HTTP ${response.status} ${errText}`.trim())
        }

        const responseData = await response.json()
        const translated = extractTranslatedText(responseData)
            .replace(/\r?\n/g, ' ')
            .replace(/^["'“”]+|["'“”]+$/g, '')
            .trim()

        if (currentToken !== translationRequestToken) return

        if (translated) {
            currentForm.value.title = translated
            if (manual) {
                utilsStore.showMessage('翻译完成', 'success')
            }
        } else if (manual) {
            utilsStore.showMessage('翻译结果为空，已保留原标题', 'warning')
        }
    } catch (error) {
        if (manual) {
            utilsStore.showMessage(`翻译失败，已保留原标题: ${error}`, 'error')
        } else {
            console.error('自动翻译失败:', error)
        }
    } finally {
        if (currentToken === translationRequestToken) {
            titleTranslating.value = false
        }
    }
}

const autoTranslateCurrentTitle = async () => {
    const config = getTranslationConfig()
    if (!config.autoTranslate) return
    await translateCurrentTitle(false)
}

const handleManualTranslateTitle = async () => {
    await translateCurrentTitle(true)
}

const getNextUploadBatch = (videos: any[]) => {
    const allVideos = Array.isArray(videos) ? videos : []
    if (!allVideos.length) {
        return {
            videos: [] as any[],
            grouped: false,
            groupKey: '',
            reason: 'empty'
        }
    }

    const templateTasks = uploadStore.uploadQueue.filter(task => {
        return (
            task.user?.uid === selectedUser.value?.uid &&
            task.template === currentTemplateName.value &&
            task.status !== 'Cancelled' &&
            task.status !== 'Failed'
        )
    })

    const queuedVideoIds = new Set(
        templateTasks
            .map(task => String(task.video?.id || ''))
            .filter(id => id.length > 0)
    )

    const pendingVideos = allVideos.filter(video => {
        const videoId = String(video?.id || '')
        if (!videoId) return false
        if (queuedVideoIds.has(videoId)) return false
        return !(video.complete && !video.path)
    })

    if (!pendingVideos.length) {
        return {
            videos: [] as any[],
            grouped: false,
            groupKey: '',
            reason: 'none_pending'
        }
    }

    const groupedByKey = new Map<string, { videos: any[]; roles: Set<string> }>()
    const orderedGroupKeys: string[] = []
    const ungroupedVideos: any[] = []

    for (const video of pendingVideos) {
        const { grouped: isGrouped, key, role } = getDerivedVideoGroup(video)

        if (!isGrouped || !role) {
            ungroupedVideos.push(video)
            continue
        }

        if (!groupedByKey.has(key)) {
            groupedByKey.set(key, { videos: [], roles: new Set<string>() })
            orderedGroupKeys.push(key)
        }
        groupedByKey.get(key)!.videos.push(video)
        groupedByKey.get(key)!.roles.add(role)
    }

    for (const groupKey of orderedGroupKeys) {
        const group = groupedByKey.get(groupKey)!
        if (group.roles.has('中配') && group.roles.has('熟肉')) {
            return {
                videos: group.videos,
                grouped: true,
                groupKey,
                reason: ''
            }
        }
    }

    if (ungroupedVideos.length > 0) {
        return {
            videos: ungroupedVideos,
            grouped: false,
            groupKey: '',
            reason: ''
        }
    }

    return {
        videos: [] as any[],
        grouped: true,
        groupKey: '',
        reason: 'incomplete_group'
    }
}

const getNextSubmitBatch = (videos: any[]) => {
    const allVideos = Array.isArray(videos) ? videos : []
    if (!allVideos.length) return [] as any[]

    const uploadedVideos = allVideos.filter(video => video.complete && video.path === '')
    if (!uploadedVideos.length) return [] as any[]

    const groupedByKey = new Map<string, { videos: any[]; roles: Set<string> }>()
    const orderedGroupKeys: string[] = []
    const uploadedUngrouped: any[] = []

    for (const video of uploadedVideos) {
        const { grouped: isGrouped, key, role } = getDerivedVideoGroup(video)

        if (!isGrouped || !role) {
            uploadedUngrouped.push(video)
            continue
        }

        if (!groupedByKey.has(key)) {
            groupedByKey.set(key, { videos: [], roles: new Set<string>() })
            orderedGroupKeys.push(key)
        }
        groupedByKey.get(key)!.videos.push(video)
        groupedByKey.get(key)!.roles.add(role)
    }

    for (const key of orderedGroupKeys) {
        const group = groupedByKey.get(key)!
        if (group.roles.has('中配') && group.roles.has('熟肉')) {
            return group.videos
        }
    }

    return uploadedUngrouped
}

const sanitizeDroppedPath = (rawPath: string) => {
    let normalized = rawPath.trim()
    if (!normalized) return ''

    if (/^file:\/\//i.test(normalized)) {
        normalized = normalized.replace(/^file:\/+/, '')
        try {
            normalized = decodeURIComponent(normalized)
        } catch {
            // ignore invalid URI encoding
        }
        if (/^\/[a-zA-Z]:/.test(normalized)) {
            normalized = normalized.slice(1)
        }
    }

    return normalized
}

const splitRawDroppedPaths = (rawValue: string): string[] =>
    rawValue
        .split(/[\r\n\0]+/)
        .map(item => item.trim())
        .filter(Boolean)

const collectDroppedPaths = (
    source: any,
    output: string[],
    visited: Set<any> = new Set<any>()
) => {
    if (source == null) return

    if (typeof source === 'string') {
        output.push(...splitRawDroppedPaths(source))
        return
    }

    if (Array.isArray(source)) {
        for (const item of source) {
            collectDroppedPaths(item, output, visited)
        }
        return
    }

    if (typeof source !== 'object') return
    if (visited.has(source)) return
    visited.add(source)

    const objectSource = source as Record<string, any>
    const directPathFields = ['path', 'filepath', 'filePath', 'fullPath', 'fullpath', 'fsPath']
    for (const field of directPathFields) {
        const fieldValue = objectSource[field]
        if (typeof fieldValue === 'string') {
            output.push(...splitRawDroppedPaths(fieldValue))
        }
    }

    const nestedFields = ['paths', 'files', 'items', 'value']
    for (const field of nestedFields) {
        if (objectSource[field] !== undefined) {
            collectDroppedPaths(objectSource[field], output, visited)
        }
    }
}

const extractDroppedVideoPaths = (videoFiles: any): string[] => {
    const collectedPaths: string[] = []
    collectDroppedPaths(videoFiles, collectedPaths)

    const uniquePaths: string[] = []
    const seenNormalizedPaths = new Set<string>()

    for (const rawPath of collectedPaths) {
        const sanitizedPath = sanitizeDroppedPath(String(rawPath || ''))
        if (!sanitizedPath) continue

        const normalizedPath = normalizeVideoPath(sanitizedPath)
        if (seenNormalizedPaths.has(normalizedPath)) continue

        seenNormalizedPaths.add(normalizedPath)
        uniquePaths.push(sanitizedPath)
    }

    return uniquePaths
}

const addVideoToCurrentForm = async (videoPath: string) => {
    const sanitizedPath = sanitizeDroppedPath(videoPath)
    if (!sanitizedPath) {
        return 0
    }

    const videoBaseName = sanitizedPath.split(/[/\\]/).pop() || sanitizedPath
    const videoNameWOExtension = videoBaseName.replace(/\.[^/.]+$/, '')
    const parsedVideoName = parseVideoNameForGrouping(videoNameWOExtension)
    const videoExt = videoBaseName.split('.').pop()?.toLowerCase() || ''

    const extFilter = [
        'mp4',
        'flv',
        'avi',
        'wmv',
        'mov',
        'webm',
        'mpeg4',
        'ts',
        'mpg',
        'rm',
        'rmvb',
        'mkv',
        'm4v'
    ]

    if (videoExt && !extFilter.includes(videoExt)) {
        return 0 // 不支持的格式，跳过添?
    }

    if (!currentForm.value) {
        return 0
    }

    const normalizedPath = normalizeVideoPath(sanitizedPath)
    const existingFile = currentForm.value.videos.find(
        f => typeof f.path === 'string' && normalizeVideoPath(f.path) === normalizedPath
    )
    if (existingFile) {
        return 0
    }

    const currentAddedVideos = currentForm.value.videos.filter(video => {
        return (
            (video.finished_at && video.finished_at > 0) || (video.path && video.path.trim() !== '')
        )
    })

    if (currentAddedVideos.length >= 100) {
        utilsStore.showMessage('单次提交最多支持 100 个视频文件，无法添加更多视频', 'error')
        return 0
    }

    const videoId = uuidv4()
    currentForm.value.videos.push({
        id: videoId,
        filename: videoBaseName,
        title: parsedVideoName.displayTitle,
        desc: '',
        path: sanitizedPath,
        complete: false,
        group_key: parsedVideoName.role ? parsedVideoName.baseTitle : '',
        group_role: parsedVideoName.role || ''
    })

    if (
        userConfigStore.configRoot?.auto_upload &&
        selectedUser.value &&
        !parsedVideoName.role &&
        !hasMatchedVideoGroups(currentForm.value.videos)
    ) {
        try {
            await uploadStore.createUploadTask(
                selectedUser.value.uid,
                currentTemplateName.value,
                currentForm.value.videos
            )

            if (userConfigStore.configRoot?.auto_start) {
                setTimeout(async () => {
                    try {
                        await autoStartWaitingTasks()
                    } catch (error) {
                        console.error('自动开始任务失败:', error)
                    }
                }, 500)
            }
        } catch (error) {
            console.error('自动添加到上传队列失败:', error)
        }
    }
    return 1
}

const handleDroppedFiles = async (videoFiles: any) => {
    if (!selectedUser.value || !currentTemplateName.value) {
        utilsStore.showMessage('请先选择用户和模板后再拖拽文件', 'warning')
        return
    }

    const droppedPaths = extractDroppedVideoPaths(videoFiles)
    if (droppedPaths.length === 0) {
        utilsStore.showMessage('未识别到可用的视频路径', 'warning')
        return
    }

    let addedCount = 0
    templateLoading.value = true
    try {
        for (const videoPath of droppedPaths) {
            addedCount += await addVideoToCurrentForm(videoPath)
        }
    } finally {
        templateLoading.value = false
    }

    if (addedCount > 0) {
        syncTitleFromGroupedVideos()
        utilsStore.showMessage(`成功添加 ${addedCount} 个视频文件`, 'success')
        await autoTranslateCurrentTitle()
    } else {
        utilsStore.showMessage('所有文件都已存在，未添加新文件', 'info')
    }
}

// ¼ɹ
const handleLoginSuccess = async () => {
    showLoginDialog.value = false
    utilsStore.showMessage('登录成功', 'success')

    await userConfigStore.saveConfig()
    // 刷新扢有数?
    await refreshAllData()
}

// 处理登录对话框关?
const handleLoginDialogClose = async (done: () => void) => {
    if (loginLoading.value) {
        utilsStore.showMessage('登录过程中无法取消', 'warning')
        return
    }

    try {
        await ElMessageBox.confirm('确定要取消登录吗？', '提示', {
            confirmButtonText: '确定',
            cancelButtonText: '继续登录',
            type: 'warning'
        })
        done()
    } catch (error) {
        // 用户点击了取消，不关闭对话框
    }
}

// 切换用户展开状?
const toggleUserExpanded = (userUid: number) => {
    userConfigStore.toggleUserExpanded(userUid)
}

// ûչť - ģʱ
const handleUserExpansion = (userUid: number) => {
    if (!templateLoading.value) {
        toggleUserExpanded(userUid)
    }
}

// ģѡ - ģʱ
const handleTemplateSelection = (user: any, templateName: string) => {
    if (!templateLoading.value) {
        selectTemplate(user, templateName)
    }
}

// 处理模板名编辑点?- ģʱ
const handleTemplateNameEdit = () => {
    if (!templateLoading.value) {
        startEditTemplateName()
    }
}

// ѡ - ģʱ
const handleCoverSelection = () => {
    if (!templateLoading.value) {
        selectCoverWithTauri()
    }
}

// ѡģ
const selectTemplate = async (user: any, templateName: string) => {
    // ڼģ壬ֹ?
    if (templateLoading.value) {
        return
    }

    if (selectedUser.value === user && currentTemplateName.value === templateName) {
        // Ѿѡͬûģ壬AҪ?
        return
    }

    templateLoading.value = true
    try {
        lastSubmit.value = ''

        selectedUser.value = user
        currentTemplateName.value = templateName

        // 滚动到顶?
        nextTick(() => {
            if (contentWrapperRef.value) {
                contentWrapperRef.value.scrollTop = 0
            }
        })

        // 加载模板数据到表?
        await loadTemplate()

        // ģѡlocalStorage
        saveTemplateSelection(user.uid, templateName)

        // 如果模板?aid，主动刷新模板数?
        const aid = currentTemplate.value?.aid
        setTimeout(async () => {
            if (aid) {
                try {
                    if (
                        selectedUser.value?.uid === user.uid &&
                        currentTemplateName.value === templateName
                    ) {
                        const newTemplate = await getNewTemplateFromAv(user.uid, aid)
                        const currentTemplateData =
                            userConfigStore.configRoot?.config[user.uid].templates[templateName]
                        if (
                            currentTemplateData &&
                            hasUnsavedChanges(currentTemplateData, newTemplate)
                        ) {
                            await ElMessageBox.confirm(
                                `棢测到本地模板内容与bilibili不一致，是否刷新？（此操作会丢失扢有未保存的更改）`,
                                '',
                                {
                                    confirmButtonText: '刷新并继续',
                                    cancelButtonText: '不刷新，仅显示当前',
                                    type: 'info'
                                }
                            )
                            await reloadTemplateFromAV(user.uid, aid)
                        }
                    }
                } catch (error) {
                    console.error('自动刷新模板数据失败:', error)
                }
            }
        }, 666)
        console.log(`лģ: ${user.username} - ${templateName}`)
    } catch (error) {
        console.error('切换模板失败:', error)
        utilsStore.showMessage(`切换模板失败: ${error}`, 'error')
    } finally {
        templateLoading.value = false
    }
}

const resetTemplate = async () => {
    if (!selectedUser.value || !currentTemplateName.value) {
        utilsStore.showMessage('请先选择用户和模板', 'warning')
        return
    }

    // ڼģ壬ֹ?
    if (templateLoading.value) {
        utilsStore.showMessage('模板正在加载中，请稍后再试', 'warning')
        return
    }

    // 确认重置
    try {
        await ElMessageBox.confirm('确定要清除所有未保存的更改吗？', '', {
            confirmButtonText: '确定',
            cancelButtonText: '取消',
            type: 'warning'
        })

        templateLoading.value = true
        try {
            currentForm.value =
                JSON.parse(
                    JSON.stringify(
                        userConfigStore.configBase?.config[selectedUser.value.uid]?.templates[
                            currentTemplateName.value
                        ]
                    )
                ) || userConfigStore.createDefaultTemplate()
            utilsStore.showMessage('模板已重置', 'success')
        } finally {
            templateLoading.value = false
        }
    } catch (error) {
        // 用户取消了重?
        console.log('重置操作已取消')
    }
}

const getNewTemplateFromAv = async (userUid: number, aid: number) => {
    try {
        const newTemplate = (await utilsStore.getVideoDetail(userUid, aid.toString())) as any

        // 处理视频列表
        if (newTemplate.videos && Array.isArray(newTemplate.videos)) {
            for (const video of newTemplate.videos) {
                video.id = video.filename
                video.path = ''
            }
        }

        if (newTemplate.aid && (await utilsStore.getSeasonList(userUid))) {
            const season_id = await utilsStore.getVideoSeason(userUid, newTemplate.aid)

            if (season_id !== 0) {
                const section_id = await utilsStore.seasonlist.find(
                    item => item.season_id === season_id
                )?.section_id
                newTemplate.season_id = season_id
                newTemplate.section_id = section_id
            }
        }

        newTemplate.watermark = currentForm.value?.watermark
        return newTemplate
    } catch (error) {
        console.error('获取新模板失败: ', error)
        throw error
    }
}

const reloadTemplateFromAV = async (userUid: number, aid: number) => {
    // ڼģ壬ֹ¼?
    if (templateLoading.value) {
        return
    }

    if (!selectedUser.value || selectedUser.value.uid !== userUid) {
        return
    }

    if (!currentForm.value || currentForm.value.aid !== aid) {
        return
    }

    templateLoading.value = true
    try {
        const newTemplate = await getNewTemplateFromAv(userUid, aid)
        currentForm.value = newTemplate
        utilsStore.showMessage('模板数据已刷新', 'success')
    } catch (error) {
        console.error('刷新模板失败: ', error)
        utilsStore.showMessage(`刷新模板失败: ${error}`, 'error')
        throw error
    } finally {
        templateLoading.value = false
    }
}

// 加载模板数据到表?
const loadTemplate = async () => {
    try {
        // ûģ壬ʹĬģ
        if (!currentTemplate.value) {
            const defaultTemplate = userConfigStore.createDefaultTemplate()
            // 直接设置到配置中
            if (
                selectedUser.value &&
                currentTemplateName.value &&
                userConfigStore.configRoot?.config
            ) {
                const userConfig = userConfigStore.configRoot.config[selectedUser.value.uid]
                if (userConfig) {
                    userConfig.templates[currentTemplateName.value] = defaultTemplate
                }
            }

            // 清空标签
            tags.value = []

            // շѡ
            selectedCategory.value = null
            selectedSubCategory.value = null

            // 等待扢有更新完?
            await nextTick()

            return
        }

        const template = currentTemplate.value

        // 解析标签
        tags.value = template.tag ? template.tag.split(',').filter(tag => tag.trim()) : []

        // 设置选中的分?
        if (template.tid) {
            setSelectedCategoryByTid(template.tid)
        } else {
            // 如果没有分区信息，清空分区择
            selectedCategory.value = null
            selectedSubCategory.value = null
        }

        // 等待扢有更新完?
        await nextTick()

        // ģֱӲ豣״?
    } catch (error) {
        console.error('加载模板失败:', error)
        utilsStore.showMessage(`加载模板失败: ${error}`, 'error')
    }
}

// ģ
const handleTemplateCommand = async (command: string, user: any, template: any) => {
    switch (command) {
        case 'duplicate':
            try {
                const newName = `${template.name}_副本`
                await userConfigStore.duplicateUserTemplate(user.uid, template.name, newName)
                utilsStore.showMessage('模板复制成功', 'success')
            } catch (error) {
                console.error('复制模板失败: ', error)
                utilsStore.showMessage(`复制模板失败: ${error}`, 'error')
            }
            break

        case 'rename':
            try {
                const { value: newName } = await ElMessageBox.prompt(
                    '请输入新的模板名',
                    '重命名模板',
                    {
                        confirmButtonText: '确定',
                        cancelButtonText: '取消',
                        inputPlaceholder: '请输入模板名',
                        inputValue: template.name,
                        inputValidator: (value: string) => {
                            if (!value || !value.trim()) {
                                return '模板名不能为空'
                            }
                            if (value.trim() === template.name) {
                                return '新名称不能与原名称相同'
                            }
                            return true
                        }
                    }
                )

                const trimmedName = newName.trim()

                // 棢查是否已存在同名模板
                const existingTemplate = userConfigStore.getUserTemplate(user.uid, trimmedName)
                if (existingTemplate) {
                    utilsStore.showMessage('该名称的模板已存在，请使用其他名称', 'error')
                    return
                }

                // 获取原模板配?
                const originalTemplate = userConfigStore.getUserTemplate(user.uid, template.name)
                if (!originalTemplate) {
                    utilsStore.showMessage('原模板不存在', 'error')
                    return
                }

                // ģ
                await userConfigStore.addUserTemplate(user.uid, trimmedName, originalTemplate)

                // ɾԭģ
                await userConfigStore.removeUserTemplate(user.uid, template.name)

                // µǰѡ
                if (
                    selectedUser.value?.uid === user.uid &&
                    currentTemplateName.value === template.name
                ) {
                    currentTemplateName.value = trimmedName
                    // 更新localStorage中的模板选择记录
                    saveTemplateSelection(user.uid, trimmedName)
                }

                utilsStore.showMessage('模板重命名成功', 'success')
            } catch (error) {
                if (error !== 'cancel') {
                    console.error('重命名模板失败: ', error)
                    utilsStore.showMessage(`重命名模板失败: ${error}`, 'error')
                }
            }
            break

        case 'delete':
            try {
                const template_name = template.name || currentTemplateName.value
                await ElMessageBox.confirm(
                    `确定要删除模板 "${template_name}" 吗？`,
                    '确认删除',
                    {
                    confirmButtonText: '确定',
                    cancelButtonText: '取消',
                    type: 'warning'
                    }
                )

                await userConfigStore.removeUserTemplate(user.uid, template_name)

                // ɾǵǰѡеģ壬ѡ
                if (
                    selectedUser.value?.uid === user.uid &&
                    currentTemplateName.value === template_name
                ) {
                    currentTemplateName.value = ''
                    selectedUser.value = null
                    // 清除localStorage中的模板选择记录
                    localStorage.removeItem(TEMPLATE_SELECTION_KEY)
                }

                utilsStore.showMessage('模板删除成功', 'success')
            } catch (error) {
                if (error !== 'cancel') {
                    console.error('删除模板失败: ', error)
                    utilsStore.showMessage(`删除模板失败: ${error}`, 'error')
                }
            }
            break
    }
}

// ģ崴ɹ¼
const handleTemplateCreated = async (userUid: number, templateName: string) => {
    if (getCurrentAutoSubmitting.value) {
        return
    }

    // 自动选择新创建的模板
    const targetUser = loginUsers.value.find(user => user.uid === userUid)
    if (targetUser) {
        selectedUser.value = targetUser
        currentTemplateName.value = templateName

        // 滚动到顶?
        nextTick(() => {
            if (contentWrapperRef.value) {
                contentWrapperRef.value.scrollTop = 0
            }
        })

        templateLoading.value = true
        await loadTemplate()
        templateLoading.value = false

        // ´ģѡ
        saveTemplateSelection(userUid, templateName)
    }
}

// ģ
const saveTemplate = async () => {
    if (!selectedUser.value || !currentTemplateName.value || !currentTemplate.value) {
        utilsStore.showMessage('请先选择模板', 'error')
        return
    }

    try {
        // ֱӱ浱ǰģ
        await userConfigStore.updateUserTemplate(
            selectedUser.value.uid,
            currentTemplateName.value,
            currentTemplate.value
        )

        // ģֱӲ棬A״?
    } catch (error) {
        console.error('保存模板失败: ', error)
        utilsStore.showMessage(`保存模板失败: ${error}`, 'error')
    }
}

// ѡ
const onCategoryChange = (categoryId: number) => {
    const category = typeList.value.find(item => item.id === categoryId)
    selectedCategory.value = category
    selectedSubCategory.value = null
    if (currentForm.value) {
        currentForm.value.tid = 0
        // һлᴥwatch事件，导致selectedCategory被清?
        nextTick(() => {
            selectedCategory.value = category
        })
    }
}

const onSubCategoryChange = (subCategoryId: number) => {
    if (selectedCategory.value && selectedCategory.value.children) {
        const subCategory = selectedCategory.value.children.find(
            (item: any) => item.id === subCategoryId
        )
        selectedSubCategory.value = subCategory
        if (currentForm.value) {
            currentForm.value.tid = subCategoryId
        }
        // ѡӷرpopover
        categoryPopoverVisible.value = false
    }
}

// 根据tid设置选中的分?
const setSelectedCategoryByTid = (tid: number) => {
    for (const category of typeList.value) {
        if (category.children) {
            const subCategory = category.children.find((item: any) => item.id === tid)
            if (subCategory) {
                selectedCategory.value = category
                selectedSubCategory.value = subCategory
                return
            }
        }
    }
}

// ѡ
const selectCoverWithTauri = async () => {
    try {
        const selected = await open({
            multiple: false,
            filters: [
                {
                    name: 'Image',
                    extensions: ['jpg', 'jpeg', 'png', 'pjp', 'pjpeg', 'jiff', 'gif']
                }
            ]
        })

        if (!selected || selected.length === 0) {
            utilsStore.showMessage('未选择任何封面文件', 'warning')
            return
        }

        if (selectedUser.value && currentTemplate.value && currentForm.value) {
            coverLoading.value = true
            templateLoading.value = true
            const url = await utilsStore.uploadCover(selectedUser.value.uid, selected)
            if (url) {
                currentTemplate.value.cover = url
                currentForm.value.cover = url
            } else {
                throw new Error('上传封面失败')
            }
        } else {
            utilsStore.showMessage('请先选择用户和模板', 'error')
        }
    } catch (error) {
        console.error('选择封面失败: ', error)
        utilsStore.showMessage(`选择封面失败: ${error}`, 'error')
        return
    } finally {
        coverLoading.value = false
        templateLoading.value = false
    }
}

// ʹ Tauri 文件对话框择视频文件
const selectVideoWithTauri = async () => {
    if (templateLoading.value) {
        utilsStore.showMessage('模板加载中，请稍后再试', 'warning')
        return
    }

    templateLoading.value = true
    try {
        const selected = await open({
            multiple: true,
            filters: [
                {
                    name: 'Video',
                    extensions: [
                        'mp4',
                        'flv',
                        'avi',
                        'wmv',
                        'mov',
                        'webm',
                        'mpeg4',
                        'ts',
                        'mpg',
                        'rm',
                        'rmvb',
                        'mkv',
                        'm4v'
                    ]
                }
            ]
        })

        let added = 0

        if (selected && Array.isArray(selected)) {
            for (const videoPath of selected) {
                added += await addVideoToCurrentForm(videoPath)
            }
        } else if (typeof selected === 'string') {
            added += await addVideoToCurrentForm(selected)
        }

        if (added > 0) {
            syncTitleFromGroupedVideos()
            utilsStore.showMessage(`已选择 ${added} 个文件`, 'success')
        }
    } catch (error) {
        console.error('选择视频失败: ', error)
        utilsStore.showMessage(`选择视频失败: ${error}`, 'error')
    } finally {
        templateLoading.value = false
    }
}

// 清空扢有文?
const clearAllVideos = async () => {
    if (!currentForm.value?.videos || currentForm.value.videos.length === 0) {
        return
    }

    const videoCount = currentForm.value.videos.length
    const videoText = videoCount === 1 ? '1 个文件' : `${videoCount} 个文件`

    templateLoading.value = true
    try {
        await ElMessageBox.confirm(`确定要清空所有已选择的${videoText}吗？`, '确认清空文件', {
            confirmButtonText: '确定清空',
            cancelButtonText: '取消',
            type: 'warning',
            dangerouslyUseHTMLString: false
        })

        // 取消扢有对应的上传任务
        const videoIds = currentForm.value.videos.map(video => video.id)
        const correspondingTasks = uploadStore.uploadQueue.filter(task =>
            videoIds.includes(task.video?.id)
        )

        for (const task of correspondingTasks) {
            try {
                await uploadStore.cancelUpload(task.id)
                console.log(`已取消对应的上传任务: ${task.id}`)
            } catch (error) {
                console.error('取消上传失败:', error)
                // 继续处理其他任务
            }
        }

        // 清空视频文件列表
        currentForm.value.videos = []
        utilsStore.showMessage(`已清空${videoText}`, 'success')
    } catch {
        // 用户取消了操?
    } finally {
        templateLoading.value = false
    }
}

// 调整视频顺序
const removeUploadedFile = async (videoId: string) => {
    if (!currentForm.value?.videos) {
        return
    }

    templateLoading.value = true
    const videoIndex = currentForm.value.videos.findIndex(f => f.id === videoId)
    if (videoIndex > -1) {
        const video = currentForm.value.videos[videoIndex]

        try {
            // 添加确认弹窗
            await ElMessageBox.confirm(
                `确定要删除视频文件 "${video.title}" 吗？此操作不可恢复。`,
                '确认删除文件',
                {
                    confirmButtonText: '确定删除',
                    cancelButtonText: '取消',
                    type: 'warning'
                }
            )

            // 先查找并取消对应的上传任?
            const correspondingTask = uploadStore.uploadQueue.find(
                task => task.video?.id === videoId
            )
            if (correspondingTask) {
                try {
                    await uploadStore.cancelUpload(correspondingTask.id)
                    console.log(`已取消对应的上传任务: ${correspondingTask.id}`)
                } catch (error) {
                    console.error('取消上传失败:', error)
                    // 即使取消失败，仍然继续删除文?
                }
            }

            // 删除视频文件
            currentForm.value.videos.splice(videoIndex, 1)

            utilsStore.showMessage('文件删除成功', 'success')
        } catch (error) {
            // 如果用户取消了确认框，不显示错误消息
            if (error !== 'cancel') {
                console.error('删除文件失败:', error)
                utilsStore.showMessage(`删除文件失败: ${error}`, 'error')
            }
        }
    }
    templateLoading.value = false
}

// ϴ
const createUpload = async () => {
    // 棢查是否有文件可上?
    const hasUploadedFiles = currentForm.value?.videos && currentForm.value.videos.length > 0

    if (!hasUploadedFiles) {
        utilsStore.showMessage('请先选择视频文件', 'error')
        return
    }

    if (!selectedUser.value) {
        utilsStore.showMessage('请选择用户', 'error')
        return
    }

    uploading.value = true
    try {
        if (currentForm.value) {
            const num_added = await uploadStore.createUploadTask(
                selectedUser.value.uid,
                currentTemplateName.value,
                currentForm.value.videos
            )
            utilsStore.showMessage(`添加 ${num_added} 个文件到上传队列`, 'success')
        }

        // 如果启用自动弢始，则自动开始任?
        if (userConfigStore.configRoot?.auto_start) {
            setTimeout(async () => {
                try {
                    await autoStartWaitingTasks()
                } catch (error) {
                    console.error('自动开始任务失败:', error)
                }
            }, 500)
        }
    } catch (error) {
        console.error('创建上传任务失败: ', error)
        utilsStore.showMessage(`创建上传任务失败: ${error}`, 'error')
    } finally {
        uploading.value = false
    }
}

// 处理文件夹监控添加视频事?
const handleAddVideosToForm = async (newVideos: any[]) => {
    templateLoading.value = true
    let addedCount = 0
    for (const videoPath of newVideos) {
        try {
            addedCount += await addVideoToCurrentForm(videoPath)
        } catch (error) {
            console.error(`添加视频失败: ${videoPath}`, error)
        }
    }
    if (addedCount > 0) {
        syncTitleFromGroupedVideos()
    }
    templateLoading.value = false
}

const handleVideosReversed = async () => {
    await nextTick()
    syncTitleFromGroupedVideos()
    await autoTranslateCurrentTitle()
}

// 处理文件夹监控提交60件事?
const handleSubmitTemplate = async () => {
    await submitTemplate()
}

// 自动弢始待处理的任?
const autoStartWaitingTasks = async () => {
    if (!userConfigStore.configRoot?.auto_start) {
        return
    }

    // 刷新上传队列获取朢新状?
    await uploadStore.getUploadQueue()

    // 获取扢有待处理的任?
    const pendingTasks = uploadStore.uploadQueue.filter(task => task.status === 'Waiting')

    for (const task of pendingTasks) {
        try {
            await uploadStore.startUpload(task.id)
            console.log(`自动弢始任? ${task.id}`)
        } catch (error) {
            console.error(`自动弢始任务失?${task.id}:`, error)
            // 继续处理下一个任?
        }
    }
}

const autoStartWaitingTasksForBatch = async (
    uid: number,
    templateName: string,
    videoIds: string[]
) => {
    if (!videoIds.length) return

    await uploadStore.getUploadQueue()
    const targetIds = new Set(videoIds)

    const otherActiveTasks = uploadStore.uploadQueue.filter(task => {
        return (
            task.user?.uid === uid &&
            task.template === templateName &&
            !targetIds.has(String(task.video?.id || '')) &&
            (task.status === 'Pending' || task.status === 'Running')
        )
    })
    for (const task of otherActiveTasks) {
        try {
            await uploadStore.pauseUpload(task.id)
            console.log(`暂停非目标分组任? ${task.id}`)
        } catch (error) {
            console.error(`暂停非目标分组任务失?${task.id}:`, error)
        }
    }

    await uploadStore.getUploadQueue()
    const pendingTasks = uploadStore.uploadQueue.filter(task => {
        return (
            task.status === 'Waiting' &&
            task.user?.uid === uid &&
            task.template === templateName &&
            targetIds.has(String(task.video?.id || ''))
        )
    })

    for (const task of pendingTasks) {
        try {
            await uploadStore.startUpload(task.id)
            console.log(`自动弢始分组任? ${task.id}`)
        } catch (error) {
            console.error(`自动弢始分组任务失?${task.id}:`, error)
        }
    }
}

// 棢查是否所有文件都已上传完?
const allFilesUploaded = computed(() => {
    if (!currentForm.value?.videos || currentForm.value.videos.length === 0) {
        return false
    }
    return currentForm.value.videos.every(video => video.complete && video.path === '')
})

// 提交视频
const submitTemplate = async () => {
    if (!currentTemplateName.value || !selectedUser.value) {
        utilsStore.showMessage('请选择模板', 'error')
        return
    }

    if (!allFilesUploaded.value) {
        const currentAutoSubmitting = getCurrentAutoSubmitting.value
        if (!currentAutoSubmitting) {
            // 首次点击，开始自动提?            // ǰvideoбupload queue
            try {
                if (currentForm.value?.videos && currentForm.value.videos.length > 0) {
                    const nextBatch = getNextUploadBatch(currentForm.value.videos)
                    if (nextBatch.videos.length === 0) {
                        if (nextBatch.reason === 'incomplete_group') {
                            utilsStore.showMessage(
                                '检测到未完整分组，请补齐中配和熟肉后再上传',
                                'warning'
                            )
                        } else {
                            utilsStore.showMessage('当前没有可上传的视频', 'info')
                        }
                        return
                    }

                    await uploadStore.createUploadTask(
                        selectedUser.value.uid,
                        currentTemplateName.value,
                        nextBatch.videos
                    )

                    autoSubmittingGroupRecord.value[
                        getTemplateKey(selectedUser.value.uid, currentTemplateName.value)
                    ] = nextBatch.videos
                        .map(video => String(video?.id || ''))
                        .filter(id => id.length > 0)

                    setTimeout(async () => {
                        try {
                            await autoStartWaitingTasksForBatch(
                                selectedUser.value.uid,
                                currentTemplateName.value,
                                autoSubmittingGroupRecord.value[
                                    getTemplateKey(selectedUser.value.uid, currentTemplateName.value)
                                ] || []
                            )
                        } catch (error) {
                            console.error('自动开始任务失败:', error)
                        }
                    }, 500)
                }
            } catch (error) {
                console.error('添加到上传队列失败', error)
                utilsStore.showMessage(`添加到上传队列失败: ${error}`, 'error')
            }
            setAutoSubmitting(selectedUser.value.uid, currentTemplateName.value, true)
            startAutoSubmitCheck()
            utilsStore.showMessage('已启动自动提交，上传完成后将自动提交', 'info')
        } else {
            // 第二次点击，取消自动提交
            setAutoSubmitting(selectedUser.value.uid, currentTemplateName.value, false)
            utilsStore.showMessage('已取消自动提交', 'info')
        }
        return
    } else {
        if (!currentForm.value) {
            utilsStore.showMessage('当前模板数据不可用', 'error')
            return
        }

        const submitVideos = getNextSubmitBatch(currentForm.value.videos)
        if (submitVideos.length === 0) {
            utilsStore.showMessage('当前没有可提交的视频', 'info')
            return
        }

        const submitForm = {
            ...currentForm.value,
            videos: submitVideos
        }
        await performTemplateSubmit(selectedUser.value.uid, currentTemplateName.value, submitForm)
    }
}

// 模板名编辑相关函?
const startEditTemplateName = () => {
    isEditingTemplateName.value = true
    editingTemplateName.value = currentTemplateName.value
    nextTick(() => {
        templateNameInputRef.value?.focus()
    })
}

const saveTemplateName = async () => {
    const newName = editingTemplateName.value.trim()

    if (!newName) {
        utilsStore.showMessage('模板名称不能为空', 'error')
        cancelEditTemplateName()
        return
    }

    if (newName === currentTemplateName.value) {
        cancelEditTemplateName()
        return
    }

    if (!selectedUser.value) {
        utilsStore.showMessage('未选择用户', 'error')
        cancelEditTemplateName()
        return
    }

    try {
        // 棢查是否已存在同名模板
        const existingTemplate = userConfigStore.getUserTemplate(selectedUser.value.uid, newName)
        if (existingTemplate) {
            utilsStore.showMessage('该名称的模板已存在，请使用其他名称', 'error')
            return
        }

        // 获取原模板配?
        const originalTemplate = userConfigStore.getUserTemplate(
            selectedUser.value.uid,
            currentTemplateName.value
        )
        if (!originalTemplate) {
            utilsStore.showMessage('原模板不存在', 'error')
            cancelEditTemplateName()
            return
        }

        // ģ
        await userConfigStore.addUserTemplate(selectedUser.value.uid, newName, originalTemplate)

        // ɾԭģ
        await userConfigStore.removeUserTemplate(selectedUser.value.uid, currentTemplateName.value)

        // µǰѡ
        currentTemplateName.value = newName

        utilsStore.showMessage('模板重命名成功', 'success')
        isEditingTemplateName.value = false
    } catch (error) {
        console.error('重命名模板失败: ', error)
        utilsStore.showMessage(`重命名模板失败: ${error}`, 'error')
        cancelEditTemplateName()
    }
}

const cancelEditTemplateName = () => {
    isEditingTemplateName.value = false
    editingTemplateName.value = ''
}

// ûط
const openUserConfig = (user: any) => {
    configUser.value = user
    userConfigVisible.value = true
}

// 棢查用户是否有上传任务
const isUserHasUploadTasks = (uid: number) => {
    return uploadStore.uploadQueue.some((task: any) => task.user?.uid === uid)
}

// ûǳ
const handleLogoutUser = async (uid: number) => {
    // 如果用户有上传任务，不允许登?
    if (isUserHasUploadTasks(uid)) {
        utilsStore.showMessage('用户有未完成的上传任务，无法登出', 'warning')
        return
    }

    try {
        const success = await authStore.logoutUser(uid)
        if (success) {
            // 如果登出的用户正是当前择的用户，清除相关记录
            if (selectedUser.value?.uid === uid) {
                selectedUser.value = null
                currentTemplateName.value = ''
                localStorage.removeItem(TEMPLATE_SELECTION_KEY)
            } else {
                // 棢查localStorage中记录的用户是否是被登出的用?
                try {
                    const saved = localStorage.getItem(TEMPLATE_SELECTION_KEY)
                    if (saved) {
                        const selection = JSON.parse(saved)
                        if (selection.userUid === uid) {
                            localStorage.removeItem(TEMPLATE_SELECTION_KEY)
                        }
                    }
                } catch (error) {
                    console.error('清理 localStorage 记录失败:', error)
                }
            }

            utilsStore.showMessage('用户已登出', 'success')
            // 刷新前端数据
            await refreshAllData()
        } else {
            utilsStore.showMessage('登出失败', 'error')
        }
    } catch (error) {
        // 如果用户取消了确认框，error会是'cancel'，不霢要显示错?
        if (error !== 'cancel') {
            console.error('登出用户失败:', error)
            utilsStore.showMessage(`登出失败: ${error}`, 'error')
        }
    }
}

// 刷新扢有数据的方法
const refreshAllData = async () => {
    try {
        // »ȡ¼û
        await authStore.getLoginUsers()
        // ¹ûģ
        await userConfigStore.buildUserTemplates(authStore.loginUsers)
        // ¼û
        await userConfigStore.loadConfig()
        // д
        await userConfigStore.saveConfig()
    } catch (error) {
        console.error('刷新数据失败:', error)
    }
}

// ־
const exportLogs = async () => {
    try {
        const zipPath = await utilsStore.exportLogs()
        const zipName = zipPath.split(/[/\\]/).pop() || zipPath

        const savePath = await save({
            defaultPath: zipName,
            filters: [{ name: 'ZIP', extensions: ['zip'] }]
        })

        if (savePath) {
            // 复制 ZIP 文件到用户指定位?
            await copyFile(zipPath, savePath)
            await remove(zipPath)
            console.log('文件已保存到:', savePath)
        }
    } catch (error) {
        console.error('导出日志失败:', error)
    }
}

// 棢查视频转码状?
const checkVideoStatus = async () => {
    if (!selectedUser.value || !currentTemplate.value?.aid) return

    try {
        // 先刷新模板数?
        await ElMessageBox.confirm(
            '此操作会重新拉取模板数据，未保存的更改将丢失，是否继续？',
            '',
            {
                confirmButtonText: '刷新并继续',
                cancelButtonText: '不刷新，仅查看当前',
                type: 'info'
            }
        )
        await reloadTemplateFromAV(selectedUser.value.uid, currentTemplate.value.aid)
        // 然后显示状对话框
        showVideoStatusDialog.value = true
    } catch (error) {
        console.error('刷新模板失败:', error)
        // 即使刷新失败也显示对话框
        showVideoStatusDialog.value = true
    }
}

// 棢查更?
const checkUpdate = async () => {
    try {
        const updateInfo = await utilsStore.checkUpdate()
        if (updateInfo) {
            // 如果有更新，显示确认对话?
            try {
                await ElMessageBox.confirm(
                    `发现新版本 ${updateInfo}，是否前往下载？`,
                    '发现更新',
                    {
                        confirmButtonText: '前往下载',
                        cancelButtonText: '稍后再说',
                        type: 'info'
                    }
                )
                // 用户确认后打弢下载页面
                await openUrl(`https://github.com/biliup/biliup-app-new/releases/tag/${updateInfo}`)
            } catch {
                // 用户取消，不做任何操?
            }
        } else {
            utilsStore.showMessage('当前已是最新版本', 'success')
        }
    } catch (error) {
        console.error('检查更新失败:', error)
    }
}
</script>

<style scoped>
.main-view {
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.header {
    background: #fff;
    border-bottom: 1px solid #e4e7ed;
    padding: 0 20px;
    position: sticky;
    top: 0;
    z-index: 100;
    flex-shrink: 0;
}

.header-content {
    display: flex;
    justify-content: space-between;
    align-items: center;
    height: 100%;
}

.app-title {
    margin: 0;
    color: #303133;
    display: inline-block;
}

.app-version {
    display: inline-block;
}

.header-center {
    display: flex;
    align-items: center;
    gap: 12px;
}

.header-right {
    display: flex;
    align-items: center;
    gap: 20px;
}

.global-config-btn {
    margin-right: 12px;
}

.main-container {
    flex: 1;
    overflow: hidden;
}

.sidebar {
    background: #f5f7fa;
    border-right: 1px solid #e4e7ed;
    padding: 20px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.sidebar-content {
    flex: 1;
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: #c1c1c1 transparent;
}

.sidebar-content::-webkit-scrollbar {
    width: 6px;
}

.sidebar-content::-webkit-scrollbar-track {
    background: transparent;
}

.sidebar-content::-webkit-scrollbar-thumb {
    background-color: #c1c1c1;
    border-radius: 3px;
}

.sidebar-content::-webkit-scrollbar-thumb:hover {
    background-color: #a8a8a8;
}

.sidebar-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
}

.sidebar-header h3 {
    margin: 0;
    color: #303133;
}

.header-buttons {
    display: flex;
    gap: 8px;
    align-items: center;
}

.highlight-checkbox {
    margin-right: 8px;
    font-size: 12px;
}

.highlight-checkbox :deep(.el-checkbox__label) {
    font-size: 12px;
    color: #606266;
}

.highlight-checkbox-text {
    line-height: 1.2;
    text-align: center;
}

.user-section {
    margin-bottom: 10px;
}

.user-header {
    display: flex;
    align-items: center;
    padding: 10px;
    background: #fff;
    border-radius: 6px;
    cursor: pointer;
    transition: background-color 0.3s;
}

.user-header:hover {
    background: #ecf5ff;
}

.user-avatar {
    margin-right: 10px;
}

.user-name {
    flex: 1;
    font-weight: 500;
}

.template-count-badge {
    margin-right: 10px;
}

.template-count-badge :deep(.el-badge__content) {
    background-color: #909399 !important;
    color: #ffffff !important;
    border: none !important;
}

.expand-icon {
    transition: transform 0.3s;
}

.expand-icon.expanded {
    transform: rotate(180deg);
}

.config-icon {
    color: #909399;
    cursor: pointer;
    margin-left: 8px;
    margin-right: 4px;
    transition: color 0.3s;
}

.config-icon:hover {
    color: #409eff;
}

.logout-icon {
    color: #f56c6c;
    cursor: pointer;
    margin-left: 8px;
    margin-right: 4px;
    transition: color 0.3s;
}

.logout-icon:hover {
    color: #f89898;
}

.logout-icon.disabled {
    color: #c0c4cc;
    cursor: not-allowed;
}

.logout-icon.disabled:hover {
    color: #c0c4cc;
}

.template-list {
    margin-left: 20px;
    margin-top: 10px;
}

.template-item {
    display: flex;
    align-items: center;
    padding: 8px 12px;
    background: #fff;
    border-radius: 4px;
    margin-bottom: 5px;
    cursor: pointer;
    transition: all 0.3s;
}

.template-item:hover {
    background: #f0f9ff;
}

.template-item.active {
    background: #ecf5ff;
    border-left: 3px solid #409eff;
}

.template-item.auto-submitting {
    position: relative;
    overflow: hidden;
    background: linear-gradient(45deg, #e3f2fd, #f3e5f5);
    border: 2px solid #409eff;
    box-shadow: 0 0 20px rgba(64, 158, 255, 0.4);
    animation: pulse-border 1.5s ease-in-out infinite alternate;
}

.template-item.auto-submitting::before {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(64, 158, 255, 0.6), transparent);
    animation: shimmer 1.5s infinite;
    z-index: 1;
}

.template-item.auto-submitting::after {
    content: '自动上传中...';
    position: absolute;
    top: 2px;
    right: 6px;
    font-size: 10px;
    color: #409eff;
    font-weight: bold;
    animation: blink 1s infinite;
    z-index: 3;
}

.template-item.auto-submitting .template-main {
    position: relative;
    z-index: 2;
}

.template-item.auto-submitting .template-name {
    color: #1976d2;
    font-weight: 600;
    text-shadow: 0 1px 3px rgba(25, 118, 210, 0.2);
}

@keyframes shimmer {
    0% {
        left: -100%;
    }
    100% {
        left: 100%;
    }
}

@keyframes pulse-border {
    0% {
        border-color: #409eff;
        box-shadow: 0 0 20px rgba(64, 158, 255, 0.4);
    }
    100% {
        border-color: #1890ff;
        box-shadow: 0 0 30px rgba(24, 144, 255, 0.6);
    }
}

@keyframes blink {
    0%,
    50% {
        opacity: 1;
    }
    51%,
    100% {
        opacity: 0.3;
    }
}

.template-item.auto-submitting-simple {
    border: 2px solid #409eff !important;
    background: rgba(64, 158, 255, 0.05) !important;
    position: relative;
}

.template-item.auto-submitting-simple::after {
    content: '自动上传中...';
    position: absolute;
    top: 2px;
    right: 6px;
    font-size: 10px;
    color: #409eff;
    font-weight: bold;
    z-index: 3;
}

.template-item.template-loading {
    position: relative;
    background: #f5f7fa;
    border: 2px solid #e4e7ed;
    cursor: not-allowed;
    opacity: 0.8;
}

.template-item.template-loading::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(230, 244, 255, 0.8), transparent);
    animation: loading-shimmer 1.5s infinite;
    z-index: 1;
}

.template-item.template-loading::after {
    content: '加载中...';
    position: absolute;
    top: 2px;
    right: 6px;
    font-size: 10px;
    color: #909399;
    font-weight: bold;
    z-index: 3;
    animation: loading-spin 1s linear infinite;
}

.template-item.template-loading .template-main {
    position: relative;
    z-index: 2;
}

@keyframes loading-shimmer {
    0% {
        transform: translateX(-100%);
    }
    100% {
        transform: translateX(100%);
    }
}

@keyframes loading-spin {
    0% {
        opacity: 1;
    }
    50% {
        opacity: 0.5;
    }
    100% {
        opacity: 1;
    }
}

.template-main {
    flex: 1;
}

.template-name {
    font-weight: 500;
    color: #303133;
    display: flex;
    align-items: center;
    gap: 6px;
}

.unsaved-indicator {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background-color: #f56c6c;
    flex-shrink: 0;
    animation: pulse-red 2s infinite;
}

@keyframes pulse-red {
    0% {
        opacity: 1;
        transform: scale(1);
    }
    50% {
        opacity: 0.7;
        transform: scale(1.1);
    }
    100% {
        opacity: 1;
        transform: scale(1);
    }
}

.template-desc {
    font-size: 12px;
    color: #909399;
    margin-top: 2px;
}

.main-content {
    padding: 0;
    overflow: hidden;
}

.content-wrapper {
    height: 100%;
    padding: 20px;
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: #c1c1c1 transparent;
}

.content-wrapper::-webkit-scrollbar {
    width: 6px;
}

.content-wrapper::-webkit-scrollbar-track {
    background: transparent;
}

.content-wrapper::-webkit-scrollbar-thumb {
    background-color: #c1c1c1;
    border-radius: 3px;
}

.content-wrapper::-webkit-scrollbar-thumb:hover {
    background-color: #a8a8a8;
}

.no-selection,
.no-template {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
}

.form-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    padding-bottom: 15px;
    border-bottom: 1px solid #e4e7ed;
}

.form-header h3 {
    margin: 0;
    color: #303133;
}

.template-name-container {
    flex: 1;
    margin-right: 20px;
}

.edit-bv-template-disaplay {
    display: inline-block;
}

.template-name-display {
    margin: 0;
    color: #303133;
    cursor: pointer;
    padding: 8px 12px;
    border-radius: 4px;
    transition: all 0.3s;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    position: relative;
    max-width: fit-content;
}

.template-name-display:hover {
    background: #f0f9ff;
    color: #409eff;
}

.template-name-display .edit-hint-icon {
    opacity: 0;
    font-size: 14px;
    transition: opacity 0.3s;
}

.template-name-display:hover .edit-hint-icon {
    opacity: 1;
}

.template-name-input {
    max-width: 300px;
}

.refresh-btn {
    cursor: pointer;
    color: #606266;
    font-size: 16px;
    transition: all 0.3s;
    border-radius: 4px;
}

.refresh-btn:hover {
    color: #409eff;
    background-color: #f0f9ff;
    transform: rotate(180deg);
}

.header-actions {
    display: flex;
    gap: 10px;
}

.form-section {
    margin-bottom: 20px;
}

.form-section.drag-target {
    border: 2px dashed #409eff;
    background: rgba(64, 158, 255, 0.05);
    transition: all 0.3s ease;
}

.form-section.drag-target .el-card__header {
    background: rgba(64, 158, 255, 0.1);
}

/* Ƭ۵ʽ */
.card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    cursor: pointer;
    user-select: none;
    transition: all 0.3s ease;
    height: 10px;
}

.card-header:hover {
    color: #409eff;
}

.card-header .header-actions {
    display: flex;
    gap: 8px;
    align-items: center;
}

.card-header .header-actions .el-button {
    margin: 0;
    padding: 4px;
    border: none;
    background: transparent;
    transition: all 0.3s ease;
}

.card-header .header-actions .el-button:hover {
    background: rgba(245, 108, 108, 0.1);
    color: #f56c6c;
    transform: scale(1.1);
}

.card-header .header-actions .el-button .el-icon {
    font-size: 14px;
}

.collapse-icon {
    transition: transform 0.3s ease;
    color: #909399;
}

.collapse-icon:hover {
    color: #409eff;
}

.collapse-icon.collapsed {
    transform: rotate(-90deg);
}

.form-section.collapsed {
    margin-bottom: 10px;
}

.card-content {
    padding-top: 0;
}

.title-input-row {
    display: flex;
    align-items: center;
    gap: 10px;
    width: 100%;
}

.title-prefix-input {
    width: 180px;
    flex: 0 0 180px;
}

.title-main-input {
    flex: 1 1 auto;
    min-width: 0;
}

.title-translate-btn {
    flex: 0 0 auto;
}

.dialog-footer {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
}

.drag-hint {
    float: right;
    color: #409eff;
    font-size: 12px;
    font-weight: 500;
}

.cover-uploader {
    position: relative;
    display: inline-block;
    z-index: 1; /* 确保容器有基硢层级 */
}

.cover-uploader .cover-image {
    width: 100px;
    height: 60px;
    object-fit: cover;
    border-radius: 4px;
    transition:
        transform 0.3s ease,
        box-shadow 0.3s ease;
    cursor: pointer;
    position: relative; /* 重要：让 z-index Ч */
}

.cover-uploader .cover-image:hover {
    transform: scale(3) translateX(25px);
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.2);
    z-index: 999; /* 确保悬浮时在朢顶层 */
    position: relative; /* 确保定位生效 */
}

.cover-uploader-icon {
    width: 100px;
    height: 60px;
    border: 1px dashed #d9d9d9;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #8c939d;
    font-size: 24px;
}

.upload-tip {
    color: #909399;
    font-size: 12px;
    margin-top: 5px;
}

.video-buttons-group {
    display: flex;
    gap: 10px;
    align-items: center;
    margin-bottom: 10px;
}

.drag-active-tip {
    color: #409eff !important;
    font-weight: 500;
    animation: pulse 1.5s infinite;
}

@keyframes pulse {
    0% {
        opacity: 1;
    }
    50% {
        opacity: 0.7;
    }
    100% {
        opacity: 1;
    }
}

.empty-users {
    text-align: center;
    margin-top: 50px;
}

/* 登录对话框样?*/
.login-dialog :deep(.el-dialog) {
    margin: 0;
    padding: 0;
    border-radius: 0;
    background: transparent;
    box-shadow: none;
    max-height: 90vh;
}

.login-dialog :deep(.el-dialog__header) {
    display: none;
}

.login-dialog :deep(.el-dialog__body) {
    padding: 0;
    max-height: 90vh;
    overflow: hidden;
}

.login-dialog-content {
    max-height: 90vh;
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: #c1c1c1 transparent;
}

.login-dialog-content::-webkit-scrollbar {
    width: 6px;
}

.login-dialog-content::-webkit-scrollbar-track {
    background: transparent;
}

.login-dialog-content::-webkit-scrollbar-thumb {
    background-color: #c1c1c1;
    border-radius: 3px;
}

.login-dialog-content::-webkit-scrollbar-thumb:hover {
    background-color: #a8a8a8;
}

.login-dialog-content .login-view {
    min-height: auto;
    padding: 0;
    background: transparent;
}

.login-dialog-content .login-container {
    max-width: none;
    width: 100%;
    padding: 0;
}

.login-dialog-content .login-card {
    margin: 0;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
    border-radius: 16px;
}

.checkbox-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

/* ʾʽ */
.form-tip {
    font-size: 12px;
    color: #909399;
    margin-top: 5px;
    line-height: 1.4;
}

.form-tip div {
    margin-bottom: 2px;
}

/* 分区选择器样?*/
.category-trigger {
    width: 100%;
    display: flex !important;
    justify-content: space-between !important;
    align-items: center !important;
    border: 1px solid #dcdfe6;
    background: #fff;
    color: #606266;
    padding: 8px 15px;
    border-radius: 4px;
    cursor: pointer;
    transition: border-color 0.3s;
    position: relative;
}

.category-trigger .category-text {
    flex: 1;
    text-align: left;
    padding-right: 30px; /* 为右侧箭头留出空?*/
}

.category-trigger:hover {
    border-color: #409eff;
}

.category-trigger.el-button--primary {
    background: #fff;
    border-color: #409eff;
    color: #409eff;
}

.category-trigger .placeholder {
    color: #c0c4cc;
}

.category-trigger .arrow-icon {
    position: absolute;
    right: 12px;
    top: 50%;
    transform: translateY(-50%);
    transition: transform 0.3s;
    flex-shrink: 0;
}

/* ѡ */
.category-selector-panel {
    display: flex;
    height: 360px;
    border-radius: 6px;
    overflow: hidden;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.category-list {
    width: 180px;
    background: #f8f9fa;
    border-right: 1px solid #e9ecef;
    overflow-y: auto;
}

.subcategory-list {
    flex: 1;
    background: #fff;
    overflow-y: auto;
}

.category-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    cursor: pointer;
    transition: all 0.3s;
    border-bottom: 1px solid #f0f2f5;
}

.category-item:hover {
    background: #e6f7ff;
    color: #1890ff;
}

.category-item.active {
    background: #1890ff;
    color: #fff;
}

.category-item.active .arrow-right {
    color: #fff;
}

.category-name {
    font-size: 13px;
}

.arrow-right {
    color: #c0c4cc;
    font-size: 12px;
    transition: color 0.3s;
}

.subcategory-item {
    padding: 12px 16px;
    cursor: pointer;
    transition: all 0.3s;
    border-bottom: 1px solid #f0f2f5;
}

.subcategory-item:hover {
    background: #f0f9ff;
}

.subcategory-item.active {
    background: #e6f7ff;
    border-left: 3px solid #1890ff;
}

.subcategory-content {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.subcategory-name {
    font-size: 14px;
    font-weight: 500;
    color: #303133;
}

.subcategory-desc {
    font-size: 12px;
    color: #909399;
    line-height: 1.4;
}

.empty-subcategory {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
}

/* 滚动条样?*/
.category-list::-webkit-scrollbar,
.subcategory-list::-webkit-scrollbar {
    width: 6px;
}

.category-list::-webkit-scrollbar-track,
.subcategory-list::-webkit-scrollbar-track {
    background: transparent;
}

.category-list::-webkit-scrollbar-thumb,
.subcategory-list::-webkit-scrollbar-thumb {
    background-color: #c1c1c1;
    border-radius: 3px;
}

.category-list::-webkit-scrollbar-thumb:hover,
.subcategory-list::-webkit-scrollbar-thumb:hover {
    background-color: #a8a8a8;
}

/* ϴ */
.upload-actions {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 16px;
    padding: 20px 0;
    margin-top: 20px;
    border-top: 1px solid #e4e7ed;
}

.upload-actions .el-button {
    min-width: 140px;
}

/* 拖拽覆盖层样?*/
.drag-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(64, 158, 255, 0.9);
    z-index: 9999;
    display: flex;
    align-items: center;
    justify-content: center;
    backdrop-filter: blur(4px);
}

.drag-content {
    text-align: center;
    color: white;
    padding: 40px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.1);
    border: 2px dashed rgba(255, 255, 255, 0.8);
    max-width: 500px;
}

.drag-icon {
    font-size: 64px;
    margin-bottom: 20px;
    animation: bounce 2s infinite;
}

@keyframes bounce {
    0%,
    20%,
    50%,
    80%,
    100% {
        transform: translateY(0);
    }
    40% {
        transform: translateY(-10px);
    }
    60% {
        transform: translateY(-5px);
    }
}

.drag-content h3 {
    margin: 0 0 10px 0;
    font-size: 24px;
    font-weight: 600;
}

.drag-content p {
    margin: 8px 0;
    font-size: 16px;
    opacity: 0.9;
}

.drag-content .warning-text {
    color: #ffd700;
    font-weight: 500;
    margin-top: 15px;
}

/* 禁用状样?*/
.cover-uploader.disabled {
    cursor: not-allowed !important;
    opacity: 0.6 !important;
}

.cover-uploader.disabled:hover {
    border-color: #dcdfe6 !important;
}

.template-name-display.disabled {
    cursor: not-allowed !important;
    opacity: 0.6 !important;
    color: #909399 !important;
}

.template-name-display.disabled .edit-hint-icon {
    color: #c0c4cc !important;
}

/* 用户头部禁用状?*/
.user-header.disabled {
    cursor: not-allowed !important;
    opacity: 0.6 !important;
}

.user-header.disabled:hover {
    background: #fff !important;
}

/* 模板项禁用状?*/
.template-item.disabled {
    cursor: not-allowed !important;
    opacity: 0.6 !important;
}

.template-item.disabled:hover {
    background: #fff !important;
}
</style>

<style>
/* 全局样式：分区择器popover */
.category-popover {
    padding: 0 !important;
}

.category-popover .el-popover__arrow {
    display: none;
}
</style>
